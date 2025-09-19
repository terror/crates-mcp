use super::*;

pub fn list_crates(path: &str) -> Result<Vec<String>> {
  let path = PathBuf::from(path);

  if !path.exists() {
    return Err(Error(anyhow!(
      "documentation directory not found at {:?}",
      path
    )));
  }

  let mut crates = fs::read_dir(&path)?
    .filter_map(|entry| entry.ok())
    .filter(|entry| entry.path().is_dir())
    .filter_map(|entry| {
      entry
        .path()
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| *name != "src" && !name.contains('.'))
        .map(|name| name.to_string())
    })
    .collect::<Vec<String>>();

  crates.sort();

  Ok(crates)
}

pub fn lookup_crate(
  request: &LookupCrateRequest,
  path: &str,
) -> Result<Documentation> {
  let path = PathBuf::from(path).join(request.name.clone());

  if !path.exists() {
    return Err(Error(anyhow!(
      "documentation not found for crate '{}' at {:?}",
      request.name,
      path
    )));
  }

  let mut items = parse_directory(&path)?;

  if let Some(ref filter_type) = request.item_type {
    items = filter_by_item_type(items, filter_type);
  }

  if let Some(ref search_query) = request.query {
    items = filter_by_query(items, search_query);
  }

  let offset = request.offset.unwrap_or(0);

  if offset > 0 && offset < items.len() {
    items = items.into_iter().skip(offset).collect();
  } else if offset >= items.len() {
    items = Vec::new();
  }

  if let Some(limit) = request.limit {
    items.truncate(limit);
  }

  Ok(Documentation {
    name: request.name.to_string(),
    items,
  })
}

fn parse_directory(dir: &Path) -> Result<Vec<Item>> {
  let entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>();

  entries?.into_iter().map(|entry| entry.path()).try_fold(
    Vec::new(),
    |mut acc, path| -> Result<Vec<Item>> {
      match (
        path.is_dir(),
        path.extension().is_some_and(|ext| ext == "html"),
      ) {
        (true, _) => acc.extend(parse_directory(&path)?),
        (false, true) => {
          if let Some(item) = parse_html_file(&path)? {
            acc.push(item);
          }
        }
        _ => {}
      }
      Ok(acc)
    },
  )
}

fn parse_html_file(file_path: &Path) -> Result<Option<Item>> {
  let document = Html::parse_document(&fs::read_to_string(file_path)?);

  let file_name = file_path
    .file_name()
    .and_then(|n| n.to_str())
    .ok_or_else(|| anyhow!("invalid file name"))
    .unwrap();

  let signature_element = document
    .select(&Selector::parse("pre.rust.item-decl").unwrap())
    .next();

  if let Some(element) = signature_element {
    let signature = html_to_text(element.inner_html());

    if signature.is_empty() {
      return Ok(None);
    }

    let description = extract_description(&document);

    let name = extract_item_name(file_name).unwrap();

    let item = match ItemKind::from(file_name) {
      ItemKind::Function => Item::Function {
        name,
        signature,
        description,
      },
      ItemKind::Struct => Item::Struct {
        name,
        signature,
        description,
        methods: extract_method_structs(&document),
      },
      ItemKind::Enum => Item::Enum {
        name,
        signature,
        description,
        variants: extract_enum_variants(&document),
      },
      ItemKind::Trait => Item::Trait {
        name,
        signature,
        description,
        methods: extract_method_structs(&document),
      },
      ItemKind::Macro => Item::Macro {
        name,
        signature,
        description,
      },
      ItemKind::Type => Item::Type {
        name,
        signature,
        description,
      },
      ItemKind::Constant => Item::Constant {
        name,
        signature,
        description,
      },
      ItemKind::Module => Item::Module {
        name,
        description,
        items: extract_module_items(&document),
      },
    };

    Ok(Some(item))
  } else {
    Ok(None)
  }
}

fn extract_item_name(file_name: &str) -> Result<String> {
  file_name
    .find('.')
    .ok_or_else(|| Error(anyhow!("invalid file name format")))
    .and_then(|dot_pos| {
      file_name[dot_pos + 1..]
        .strip_suffix(".html")
        .ok_or_else(|| Error(anyhow!("file name doesn't end with .html")))
        .map(|name| name.to_string())
    })
}

fn extract_method_structs(document: &Html) -> Vec<Method> {
  document
    .select(&Selector::parse("div.impl-items .method").unwrap())
    .filter_map(|method_element| {
      let signature_element = method_element
        .select(&Selector::parse(".code-header").unwrap())
        .next()
        .unwrap();

      let signature = html_to_text(signature_element.inner_html());

      if signature.is_empty() {
        return None;
      }

      let name = extract_method_name(&signature);

      let description = method_element
        .select(&Selector::parse(".docblock").unwrap())
        .next()
        .map(|desc| html_to_text(desc.inner_html()))
        .filter(|text| !text.trim().is_empty())
        .map(|text| text.trim().to_string());

      Some(Method {
        name,
        signature,
        description,
      })
    })
    .collect()
}

fn extract_method_name(signature: &str) -> String {
  signature
    .strip_prefix("fn ")
    .and_then(|after_fn| {
      after_fn
        .split_once('(')
        .map(|(name, _)| name.trim().to_string())
    })
    .unwrap_or_else(|| "unknown".to_string())
}

fn extract_enum_variants(document: &Html) -> Vec<String> {
  document
    .select(&Selector::parse("div.variants .variant").unwrap())
    .filter_map(|variant_element| {
      variant_element
        .select(&Selector::parse(".code-header").unwrap())
        .next()
        .map(|header| html_to_text(header.inner_html()))
        .filter(|variant| !variant.is_empty())
    })
    .collect()
}

fn extract_module_items(document: &Html) -> Vec<String> {
  document
    .select(&Selector::parse("div.item-table .item-name a").unwrap())
    .map(|link| html_to_text(link.inner_html()))
    .filter(|item| !item.is_empty())
    .collect()
}

fn extract_description(document: &Html) -> Option<String> {
  document
    .select(&Selector::parse("details.toggle.top-doc div.docblock").unwrap())
    .next()
    .map(|element| html_to_text(element.inner_html()))
    .filter(|text| !text.trim().is_empty())
    .map(|text| text.trim().to_string())
}

fn filter_by_item_type(items: Vec<Item>, filter_type: &str) -> Vec<Item> {
  items
    .into_iter()
    .filter(|item| {
      match item {
        Item::Function { .. } => "function",
        Item::Struct { .. } => "struct",
        Item::Enum { .. } => "enum",
        Item::Trait { .. } => "trait",
        Item::Macro { .. } => "macro",
        Item::Type { .. } => "type",
        Item::Constant { .. } => "constant",
        Item::Module { .. } => "module",
      }
      .eq_ignore_ascii_case(filter_type)
    })
    .collect()
}

fn filter_by_query(items: Vec<Item>, query: &str) -> Vec<Item> {
  let query_lower = query.to_lowercase();

  items
    .into_iter()
    .filter(|item| {
      let (name, description) = item.search_items();

      name.to_lowercase().contains(&query_lower)
        || description.as_ref().is_some_and(|description| {
          description.to_lowercase().contains(&query_lower)
        })
    })
    .collect()
}

fn html_to_text(html: String) -> String {
  [
    |text: String| {
      Regex::new(r"<[^>]*>")
        .unwrap()
        .replace_all(&text, "")
        .into_owned()
    },
    |text: String| {
      [
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#39;", "'"),
      ]
      .iter()
      .fold(text, |acc, &(entity, replacement)| {
        acc.replace(entity, replacement)
      })
    },
    |text: String| {
      Regex::new(r"\s+")
        .unwrap()
        .replace_all(&text, " ")
        .trim()
        .to_string()
    },
  ]
  .iter()
  .fold(html, |text, transform| transform(text))
}

#[cfg(test)]
mod tests {
  use {super::*, std::fs, tempfile::TempDir};

  fn initialize(temp_dir: &TempDir) -> String {
    let doc_path = temp_dir.path().join("target").join("doc");
    fs::create_dir_all(&doc_path).unwrap();
    fs::create_dir_all(doc_path.join("crate")).unwrap();
    doc_path.to_string_lossy().to_string()
  }

  fn function_html(
    name: &str,
    signature: &str,
    description: Option<&str>,
  ) -> String {
    let desc_html = description
      .map(|d| format!(r#"<details class="toggle top-doc" open><summary class="hideme"><span>Expand description</span></summary><div class="docblock"><p>{}</p></div></details>"#, d))
      .unwrap_or_default();

    format!(
      r#"<!DOCTYPE html><html><head><title>{}</title></head><body>
      <pre class="rust item-decl"><code>{}</code></pre>
      {}
      </body></html>"#,
      name, signature, desc_html
    )
  }

  fn struct_html(
    name: &str,
    signature: &str,
    description: Option<&str>,
    methods: &[(String, String, Option<String>)],
  ) -> String {
    let desc_html = description
      .map(|d| format!(r#"<details class="toggle top-doc" open><summary class="hideme"><span>Expand description</span></summary><div class="docblock"><p>{}</p></div></details>"#, d))
      .unwrap_or_default();

    let methods_html = methods
      .iter()
      .map(|(_method_name, method_sig, method_desc)| {
        let method_desc_html = method_desc
          .as_ref()
          .map(|d| format!(r#"<div class="docblock"><p>{}</p></div>"#, d))
          .unwrap_or_default();
        format!(
          r#"<div class="method"><div class="code-header">{}</div>{}</div>"#,
          method_sig, method_desc_html
        )
      })
      .collect::<Vec<_>>()
      .join("");

    format!(
      r#"<!DOCTYPE html><html><head><title>{}</title></head><body>
      <pre class="rust item-decl"><code>{}</code></pre>
      {}
      <div class="impl-items">{}</div>
      </body></html>"#,
      name, signature, desc_html, methods_html
    )
  }

  fn enum_html(
    name: &str,
    signature: &str,
    description: Option<&str>,
    variants: &[String],
  ) -> String {
    let desc_html = description
      .map(|d| format!(r#"<details class="toggle top-doc" open><summary class="hideme"><span>Expand description</span></summary><div class="docblock"><p>{}</p></div></details>"#, d))
      .unwrap_or_default();

    let variants_html = variants
      .iter()
      .map(|variant| {
        format!(
          r#"<div class="variant"><div class="code-header">{}</div></div>"#,
          variant
        )
      })
      .collect::<Vec<_>>()
      .join("");

    format!(
      r#"<!DOCTYPE html><html><head><title>{}</title></head><body>
      <pre class="rust item-decl"><code>{}</code></pre>
      {}
      <div class="variants">{}</div>
      </body></html>"#,
      name, signature, desc_html, variants_html
    )
  }

  fn module_html(
    name: &str,
    description: Option<&str>,
    items: &[String],
  ) -> String {
    let desc_html = description
      .map(|d| format!(r#"<details class="toggle top-doc" open><summary class="hideme"><span>Expand description</span></summary><div class="docblock"><p>{}</p></div></details>"#, d))
      .unwrap_or_default();

    let items_html = items
      .iter()
      .map(|item| {
        format!("<div class=\"item-name\"><a href=\"#\">{}</a></div>", item)
      })
      .collect::<Vec<_>>()
      .join("");

    format!(
      r#"<!DOCTYPE html><html><head><title>{}</title></head><body>
      <pre class="rust item-decl"><code>mod {}</code></pre>
      {}
      <div class="item-table">{}</div>
      </body></html>"#,
      name, name, desc_html, items_html
    )
  }

  #[test]
  fn list_crates() {
    let temp_dir = TempDir::new().unwrap();

    let doc_path = initialize(&temp_dir);

    fs::create_dir_all(temp_dir.path().join("target/doc/crate_a")).unwrap();
    fs::create_dir_all(temp_dir.path().join("target/doc/crate_b")).unwrap();
    fs::create_dir_all(temp_dir.path().join("target/doc/crate_c")).unwrap();

    fs::create_dir_all(temp_dir.path().join("target/doc/static.files"))
      .unwrap();

    let crates = super::list_crates(&doc_path).unwrap();

    assert_eq!(crates, vec!["crate", "crate_a", "crate_b", "crate_c"]);
  }

  #[test]
  fn parse_function() {
    let temp_dir = TempDir::new().unwrap();

    let doc_path = initialize(&temp_dir);

    let crate_path = temp_dir.path().join("target/doc/crate");

    let function_html = function_html(
      "add",
      "pub fn add(a: i32, b: i32) -> i32",
      Some("Adds two numbers together."),
    );

    fs::write(crate_path.join("fn.add.html"), function_html).unwrap();

    let request = LookupCrateRequest {
      name: "crate".to_string(),
      item_type: None,
      query: None,
      limit: None,
      offset: None,
    };

    let result = lookup_crate(&request, &doc_path).unwrap();

    assert_eq!(
      result.items,
      vec![Item::Function {
        name: "add".to_string(),
        signature: "pub fn add(a: i32, b: i32) -> i32".to_string(),
        description: Some("Adds two numbers together.".to_string()),
      }]
    );
  }

  #[test]
  fn parse_struct_with_methods() {
    let temp_dir = TempDir::new().unwrap();

    let doc_path = initialize(&temp_dir);

    let crate_path = temp_dir.path().join("target/doc/crate");

    let methods = vec![
      (
        "new".to_string(),
        "fn new() -> Self".to_string(),
        Some("Creates a new instance.".to_string()),
      ),
      (
        "get_value".to_string(),
        "fn get_value(&self) -> i32".to_string(),
        None,
      ),
    ];

    let struct_html = struct_html(
      "MyStruct",
      "pub struct MyStruct { value: i32 }",
      Some("A simple struct with a value."),
      &methods,
    );

    fs::write(crate_path.join("struct.MyStruct.html"), struct_html).unwrap();

    let request = LookupCrateRequest {
      name: "crate".to_string(),
      item_type: None,
      query: None,
      limit: None,
      offset: None,
    };

    let result = lookup_crate(&request, &doc_path).unwrap();

    assert_eq!(
      result.items,
      vec![Item::Struct {
        name: "MyStruct".to_string(),
        signature: "pub struct MyStruct { value: i32 }".to_string(),
        description: Some("A simple struct with a value.".to_string()),
        methods: vec![
          Method {
            name: "new".to_string(),
            signature: "fn new() -> Self".to_string(),
            description: Some("Creates a new instance.".to_string()),
          },
          Method {
            name: "get_value".to_string(),
            signature: "fn get_value(&self) -> i32".to_string(),
            description: None,
          },
        ],
      }]
    );
  }

  #[test]
  fn parse_enum_with_variants() {
    let temp_dir = TempDir::new().unwrap();

    let doc_path = initialize(&temp_dir);

    let crate_path = temp_dir.path().join("target/doc/crate");

    let variants = vec!["Success".to_string(), "Error(String)".to_string()];

    let enum_html = enum_html(
      "Result",
      "pub enum Result",
      Some("A type representing either success or failure."),
      &variants,
    );

    fs::write(crate_path.join("enum.Result.html"), enum_html).unwrap();

    let request = LookupCrateRequest {
      name: "crate".to_string(),
      item_type: None,
      query: None,
      limit: None,
      offset: None,
    };

    let result = lookup_crate(&request, &doc_path).unwrap();

    assert_eq!(
      result.items,
      vec![Item::Enum {
        name: "Result".to_string(),
        signature: "pub enum Result".to_string(),
        description: Some(
          "A type representing either success or failure.".to_string()
        ),
        variants: vec!["Success".to_string(), "Error(String)".to_string()],
      }]
    );
  }

  #[test]
  fn parse_module() {
    let temp_dir = TempDir::new().unwrap();

    let doc_path = initialize(&temp_dir);

    let crate_path = temp_dir.path().join("target/doc/crate");

    let items = vec!["function_a".to_string(), "struct_b".to_string()];

    let module_html =
      module_html("utils", Some("Utility functions and types."), &items);

    fs::write(crate_path.join("module.index.html"), module_html).unwrap();

    let request = LookupCrateRequest {
      name: "crate".to_string(),
      item_type: None,
      query: None,
      limit: None,
      offset: None,
    };

    let result = lookup_crate(&request, &doc_path).unwrap();

    assert_eq!(
      result.items,
      vec![Item::Module {
        name: "index".to_string(),
        description: Some("Utility functions and types.".to_string()),
        items: vec!["function_a".to_string(), "struct_b".to_string()],
      }]
    );
  }

  #[test]
  fn filter_by_item_type() {
    let temp_dir = TempDir::new().unwrap();

    let doc_path = initialize(&temp_dir);

    let crate_path = temp_dir.path().join("target/doc/crate");

    let function_html =
      function_html("add", "pub fn add(a: i32, b: i32) -> i32", None);

    let struct_html = struct_html("MyStruct", "pub struct MyStruct", None, &[]);

    fs::write(crate_path.join("fn.add.html"), function_html).unwrap();
    fs::write(crate_path.join("struct.MyStruct.html"), struct_html).unwrap();

    let request = LookupCrateRequest {
      name: "crate".to_string(),
      item_type: Some("function".to_string()),
      query: None,
      limit: None,
      offset: None,
    };

    let result = lookup_crate(&request, &doc_path).unwrap();

    assert_eq!(
      result.items,
      vec![Item::Function {
        name: "add".to_string(),
        signature: "pub fn add(a: i32, b: i32) -> i32".to_string(),
        description: None,
      }]
    );
  }

  #[test]
  fn filter_by_query() {
    let temp_dir = TempDir::new().unwrap();

    let doc_path = initialize(&temp_dir);

    let crate_path = temp_dir.path().join("target/doc/crate");

    let function1_html = function_html(
      "add",
      "pub fn add(a: i32, b: i32) -> i32",
      Some("Adds numbers"),
    );

    let function2_html = function_html(
      "subtract",
      "pub fn subtract(a: i32, b: i32) -> i32",
      Some("Subtracts numbers"),
    );

    fs::write(crate_path.join("fn.add.html"), function1_html).unwrap();
    fs::write(crate_path.join("fn.subtract.html"), function2_html).unwrap();

    let request = LookupCrateRequest {
      name: "crate".to_string(),
      item_type: None,
      query: Some("add".to_string()),
      limit: None,
      offset: None,
    };

    let result = lookup_crate(&request, &doc_path).unwrap();

    assert_eq!(
      result.items,
      vec![Item::Function {
        name: "add".to_string(),
        signature: "pub fn add(a: i32, b: i32) -> i32".to_string(),
        description: Some("Adds numbers".to_string()),
      }]
    );
  }

  #[test]
  fn pagination() {
    let temp_dir = TempDir::new().unwrap();

    let doc_path = initialize(&temp_dir);

    let crate_path = temp_dir.path().join("target/doc/crate");

    for i in 0..5 {
      let function_html = function_html(
        &format!("func_{}", i),
        &format!("pub fn func_{}()", i),
        None,
      );

      fs::write(
        crate_path.join(format!("fn.func_{}.html", i)),
        function_html,
      )
      .unwrap();
    }

    let request = LookupCrateRequest {
      name: "crate".to_string(),
      item_type: None,
      query: None,
      limit: Some(2),
      offset: Some(1),
    };

    let result = lookup_crate(&request, &doc_path).unwrap();

    assert_eq!(result.items.len(), 2);
  }

  #[test]
  fn extract_item_name() {
    assert_eq!(super::extract_item_name("fn.add.html").unwrap(), "add");

    assert_eq!(
      super::extract_item_name("struct.MyStruct.html").unwrap(),
      "MyStruct"
    );

    assert_eq!(
      super::extract_item_name("enum.Result.html").unwrap(),
      "Result"
    );

    assert_eq!(
      super::extract_item_name("module.index.html").unwrap(),
      "index"
    );

    assert!(super::extract_item_name("invalid").is_err());

    assert!(super::extract_item_name("fn.add.txt").is_err());
  }

  #[test]
  fn extract_method_name() {
    assert_eq!(super::extract_method_name("fn new() -> Self"), "new");

    assert_eq!(
      super::extract_method_name("fn get_value(&self) -> i32"),
      "get_value"
    );

    assert_eq!(
      super::extract_method_name(
        "fn complex_method<T>(self, param: T) where T: Clone"
      ),
      "complex_method<T>"
    );

    assert_eq!(super::extract_method_name("invalid signature"), "unknown");
  }

  #[test]
  fn html_to_text() {
    assert_eq!(
      super::html_to_text("<p>Hello <strong>world</strong></p>".to_string()),
      "Hello world"
    );

    assert_eq!(
      super::html_to_text("&lt;div&gt; &amp; &quot;test&quot;".to_string()),
      "<div> & \"test\""
    );

    assert_eq!(
      super::html_to_text("   Multiple    spaces   ".to_string()),
      "Multiple spaces"
    );

    assert_eq!(
      super::html_to_text("<code>fn test() -&gt; bool</code>".to_string()),
      "fn test() -> bool"
    );
  }

  #[test]
  fn nested_directories() {
    let temp_dir = TempDir::new().unwrap();

    let doc_path = initialize(&temp_dir);

    let crate_path = temp_dir.path().join("target/doc/crate");

    let nested_path = crate_path.join("submodule");
    fs::create_dir_all(&nested_path).unwrap();

    let function_html =
      function_html("nested_func", "pub fn nested_func()", None);

    fs::write(nested_path.join("fn.nested_func.html"), function_html).unwrap();

    let request = LookupCrateRequest {
      name: "crate".to_string(),
      item_type: None,
      query: None,
      limit: None,
      offset: None,
    };

    let result = lookup_crate(&request, &doc_path).unwrap();

    assert_eq!(
      result.items,
      vec![Item::Function {
        name: "nested_func".to_string(),
        signature: "pub fn nested_func()".to_string(),
        description: None,
      }]
    );
  }
}
