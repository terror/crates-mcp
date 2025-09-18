use super::*;

const DOC_PATH: &str = "target/doc";

#[derive(Debug, Serialize, Deserialize)]
pub struct Documentation {
  pub name: String,
  pub items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
  pub signature: String,
  pub description: Option<String>,
  pub methods: Vec<String>,
}

pub fn list_crates() -> Result<Vec<String>> {
  let path = PathBuf::from(DOC_PATH);

  if !path.exists() {
    bail!("documentation directory not found at {:?}", path);
  }

  let mut crates = Vec::new();

  for entry in fs::read_dir(&path)? {
    let entry = entry?;

    let path = entry.path();

    if path.is_dir() {
      match path.file_name().and_then(|n| n.to_str()) {
        Some(crate_name) if !crate_name.contains('.') => {
          crates.push(crate_name.to_string());
        }
        _ => continue,
      }
    }
  }

  crates.sort();

  Ok(crates)
}

pub fn lookup_crate(name: &str) -> Result<Documentation> {
  let path = PathBuf::from(DOC_PATH).join(name);

  if !path.exists() {
    bail!("Documentation not found for crate '{}' at {:?}", name, path);
  }

  let mut items = Vec::new();

  parse_directory(&path, &mut items)?;

  Ok(Documentation {
    name: name.to_string(),
    items,
  })
}

fn parse_directory(dir: &Path, items: &mut Vec<Item>) -> Result<()> {
  for entry in fs::read_dir(dir)? {
    let entry = entry?;

    let path = entry.path();

    if path.is_dir() {
      parse_directory(&path, items)?;
    } else if path.extension().map_or(false, |ext| ext == "html") {
      parse_html_file(&path, items)?;
    }
  }

  Ok(())
}

fn parse_html_file(file_path: &Path, items: &mut Vec<Item>) -> Result<()> {
  let document = Html::parse_document(&fs::read_to_string(file_path)?);

  let item_selector = Selector::parse("pre.rust.item-decl").unwrap();

  if let Some(item_element) = document.select(&item_selector).next() {
    let signature = clean_html_to_text(item_element.inner_html());

    if !signature.is_empty() {
      let description = extract_description(&document);

      let mut methods = Vec::new();

      let method_selector =
        Selector::parse("div.impl-items .method .code-header").unwrap();

      for method_element in document.select(&method_selector) {
        let method_signature = clean_html_to_text(method_element.inner_html());

        if !method_signature.is_empty() {
          methods.push(method_signature);
        }
      }

      items.push(Item {
        signature,
        description,
        methods,
      });
    }
  }

  Ok(())
}

fn extract_description(document: &Html) -> Option<String> {
  let selector =
    Selector::parse("details.toggle.top-doc div.docblock").unwrap();

  if let Some(element) = document.select(&selector).next() {
    let text = clean_html_to_text(element.inner_html());

    if text.trim().is_empty() {
      None
    } else {
      Some(text.trim().to_string())
    }
  } else {
    None
  }
}

fn clean_html_to_text(html: String) -> String {
  let tag_regex = Regex::new(r"<[^>]*>").unwrap();

  let cleaned = tag_regex.replace_all(&html, "");

  let cleaned = cleaned
    .replace("&amp;", "&")
    .replace("&lt;", "<")
    .replace("&gt;", ">")
    .replace("&quot;", "\"")
    .replace("&#39;", "'");

  let whitespace_regex = Regex::new(r"\s+").unwrap();

  let cleaned = whitespace_regex.replace_all(&cleaned, " ");

  cleaned.trim().to_string()
}
