use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Method {
  pub name: String,
  pub signature: String,
  pub description: Option<String>,
}
