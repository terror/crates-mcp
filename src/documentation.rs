use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Documentation {
  pub name: String,
  pub items: Vec<Item>,
}
