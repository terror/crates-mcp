use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
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

impl Item {
  pub fn search_items(&self) -> (&String, &Option<String>) {
    match self {
      Self::Function {
        name, description, ..
      } => (name, description),
      Self::Struct {
        name, description, ..
      } => (name, description),
      Self::Enum {
        name, description, ..
      } => (name, description),
      Self::Trait {
        name, description, ..
      } => (name, description),
      Self::Macro {
        name, description, ..
      } => (name, description),
      Self::Type {
        name, description, ..
      } => (name, description),
      Self::Constant {
        name, description, ..
      } => (name, description),
      Self::Module {
        name, description, ..
      } => (name, description),
    }
  }
}
