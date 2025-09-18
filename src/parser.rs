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

pub fn lookup_crate(name: &str) -> Result<Documentation> {
  let path = PathBuf::from(DOC_PATH).join(name);

  if !path.exists() {
    bail!("documentation not found for crate '{}' at {:?}", name, path);
  }

  Ok(Documentation {
    name: name.to_string(),
    items: parse_directory(&path)?,
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

  document
    .select(&Selector::parse("pre.rust.item-decl").unwrap())
    .next()
    .map(|item_element| html_to_text(item_element.inner_html()))
    .filter(|signature| !signature.is_empty())
    .map(|signature| Item {
      signature,
      description: extract_description(&document),
      methods: extract_methods(&document),
    })
    .map(Some)
    .ok_or_else(|| anyhow!("no valid item found"))
    .or(Ok(None))
}

fn extract_methods(document: &Html) -> Vec<String> {
  document
    .select(&Selector::parse("div.impl-items .method .code-header").unwrap())
    .map(|method_element| html_to_text(method_element.inner_html()))
    .filter(|method_signature| !method_signature.is_empty())
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
