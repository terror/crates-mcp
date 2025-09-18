use super::*;

const DOC_PATH: &str = "target/doc";

#[derive(Debug, Serialize, Deserialize)]
pub struct Documentation {
  pub name: String,
  pub items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Item {
  Function {
    name: String,
    signature: String,
    description: Option<String>,
  },
  Struct {
    name: String,
    signature: String,
    description: Option<String>,
    methods: Vec<Method>,
  },
  Enum {
    name: String,
    signature: String,
    description: Option<String>,
    variants: Vec<String>,
  },
  Trait {
    name: String,
    signature: String,
    description: Option<String>,
    methods: Vec<Method>,
  },
  Macro {
    name: String,
    signature: String,
    description: Option<String>,
  },
  Type {
    name: String,
    signature: String,
    description: Option<String>,
  },
  Constant {
    name: String,
    signature: String,
    description: Option<String>,
  },
  Module {
    name: String,
    description: Option<String>,
    items: Vec<String>,
  },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Method {
  pub name: String,
  pub signature: String,
  pub description: Option<String>,
}

pub fn list_crates() -> Result<Vec<String>> {
  let path = PathBuf::from(DOC_PATH);

  if !path.exists() {
    bail!("documentation directory not found at {:?}", path);
  }

  let mut crates = fs::read_dir(&path)?
    .filter_map(|entry| entry.ok())
    .filter(|entry| entry.path().is_dir())
    .filter_map(|entry| {
      entry
        .path()
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.contains('.'))
        .map(|name| name.to_string())
    })
    .collect::<Vec<String>>();

  crates.sort();

  Ok(crates)
}

pub fn lookup_crate_with_options(
  name: &str,
  limit: Option<usize>,
  offset: Option<usize>,
  item_type: Option<String>,
  query: Option<String>,
) -> Result<Documentation> {
  let path = PathBuf::from(DOC_PATH).join(name);

  if !path.exists() {
    bail!("documentation not found for crate '{}' at {:?}", name, path);
  }

  let mut items = parse_directory(&path)?;

  if let Some(ref filter_type) = item_type {
    items = filter_by_item_type(items, filter_type);
  }

  if let Some(ref search_query) = query {
    items = filter_by_query(items, search_query);
  }

  let offset = offset.unwrap_or(0);

  if offset > 0 && offset < items.len() {
    items = items.into_iter().skip(offset).collect();
  } else if offset >= items.len() {
    items = Vec::new();
  }

  if let Some(limit) = limit {
    items.truncate(limit);
  }

  Ok(Documentation {
    name: name.to_string(),
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
        path.extension().map_or(false, |ext| ext == "html"),
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
    .ok_or_else(|| anyhow!("invalid file name"))?;

  let signature_element = document
    .select(&Selector::parse("pre.rust.item-decl").unwrap())
    .next();

  if let Some(element) = signature_element {
    let signature = html_to_text(element.inner_html());

    if signature.is_empty() {
      return Ok(None);
    }

    let description = extract_description(&document);

    let name = extract_item_name(file_name)?;

    let item = match determine_item_type(file_name) {
      ItemType::Function => Item::Function {
        name,
        signature,
        description,
      },
      ItemType::Struct => Item::Struct {
        name,
        signature,
        description,
        methods: extract_method_structs(&document),
      },
      ItemType::Enum => Item::Enum {
        name,
        signature,
        description,
        variants: extract_enum_variants(&document),
      },
      ItemType::Trait => Item::Trait {
        name,
        signature,
        description,
        methods: extract_method_structs(&document),
      },
      ItemType::Macro => Item::Macro {
        name,
        signature,
        description,
      },
      ItemType::Type => Item::Type {
        name,
        signature,
        description,
      },
      ItemType::Constant => Item::Constant {
        name,
        signature,
        description,
      },
      ItemType::Module => Item::Module {
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

enum ItemType {
  Function,
  Struct,
  Enum,
  Trait,
  Macro,
  Type,
  Constant,
  Module,
}

fn determine_item_type(file_name: &str) -> ItemType {
  if file_name.starts_with("fn.") {
    ItemType::Function
  } else if file_name.starts_with("struct.") {
    ItemType::Struct
  } else if file_name.starts_with("enum.") {
    ItemType::Enum
  } else if file_name.starts_with("trait.") {
    ItemType::Trait
  } else if file_name.starts_with("macro.") {
    ItemType::Macro
  } else if file_name.starts_with("type.") {
    ItemType::Type
  } else if file_name.starts_with("constant.") {
    ItemType::Constant
  } else {
    ItemType::Module
  }
}

fn extract_item_name(file_name: &str) -> Result<String> {
  if let Some(dot_pos) = file_name.find('.') {
    let remaining = &file_name[dot_pos + 1..];

    if let Some(html_pos) = remaining.rfind(".html") {
      Ok(remaining[..html_pos].to_string())
    } else {
      bail!("file name doesn't end with .html")
    }
  } else {
    bail!("invalid file name format")
  }
}

fn extract_method_structs(document: &Html) -> Vec<Method> {
  document
    .select(&Selector::parse("div.impl-items .method").unwrap())
    .filter_map(|method_element| {
      let signature_element = method_element
        .select(&Selector::parse(".code-header").unwrap())
        .next()?;

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
  if let Some(fn_pos) = signature.find("fn ") {
    let after_fn = &signature[fn_pos + 3..];

    if let Some(paren_pos) = after_fn.find('(') {
      after_fn[..paren_pos].trim().to_string()
    } else {
      "unknown".to_string()
    }
  } else {
    "unknown".to_string()
  }
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
      let item_type = match item {
        Item::Function { .. } => "function",
        Item::Struct { .. } => "struct",
        Item::Enum { .. } => "enum",
        Item::Trait { .. } => "trait",
        Item::Macro { .. } => "macro",
        Item::Type { .. } => "type",
        Item::Constant { .. } => "constant",
        Item::Module { .. } => "module",
      };
      item_type.eq_ignore_ascii_case(filter_type)
    })
    .collect()
}

fn filter_by_query(items: Vec<Item>, query: &str) -> Vec<Item> {
  let query_lower = query.to_lowercase();

  items
    .into_iter()
    .filter(|item| {
      let (name, description) = match item {
        Item::Function {
          name, description, ..
        } => (name, description),
        Item::Struct {
          name, description, ..
        } => (name, description),
        Item::Enum {
          name, description, ..
        } => (name, description),
        Item::Trait {
          name, description, ..
        } => (name, description),
        Item::Macro {
          name, description, ..
        } => (name, description),
        Item::Type {
          name, description, ..
        } => (name, description),
        Item::Constant {
          name, description, ..
        } => (name, description),
        Item::Module {
          name, description, ..
        } => (name, description),
      };

      if name.to_lowercase().contains(&query_lower) {
        return true;
      }

      if let Some(desc) = description {
        if desc.to_lowercase().contains(&query_lower) {
          return true;
        }
      }

      false
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
