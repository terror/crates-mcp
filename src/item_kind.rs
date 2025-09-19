use super::*;

#[derive(Debug, Deserialize, Serialize)]
pub enum ItemKind {
  Function,
  Struct,
  Enum,
  Trait,
  Macro,
  Type,
  Constant,
  Module,
}

impl From<&str> for ItemKind {
  fn from(value: &str) -> Self {
    match value {
      v if v.starts_with("fn.") => Self::Function,
      v if v.starts_with("struct.") => Self::Struct,
      v if v.starts_with("enum.") => Self::Enum,
      v if v.starts_with("trait.") => Self::Trait,
      v if v.starts_with("macro.") => Self::Macro,
      v if v.starts_with("type.") => Self::Type,
      v if v.starts_with("constant.") => Self::Constant,
      _ => Self::Module,
    }
  }
}
