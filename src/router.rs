use super::*;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListCratesRequest {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LookupCrateRequest {
  #[schemars(description = "The name of the Rust crate")]
  name: String,
}

#[derive(Debug, Clone)]
pub struct Router {
  tool_router: ToolRouter<Self>,
}

impl Default for Router {
  fn default() -> Self {
    Self::new()
  }
}

#[tool_handler]
impl ServerHandler for Router {
  fn get_info(&self) -> ServerInfo {
    ServerInfo {
      capabilities: ServerCapabilities::builder().enable_tools().build(),
      instructions: Some("Find information about relevant Rust crates".into()),
      ..Default::default()
    }
  }
}

#[tool_router]
impl Router {
  pub fn new() -> Self {
    Self {
      tool_router: Self::tool_router(),
    }
  }

  #[tool(description = "List all available Rust crates")]
  fn list_crates(
    &self,
    Parameters(ListCratesRequest {}): Parameters<ListCratesRequest>,
  ) -> Result<CallToolResult, McpError> {
    match list_crates() {
      Ok(crates) => Ok(CallToolResult::success(vec![Content::text(
        crates.join("\n"),
      )])),
      Err(error) => Ok(CallToolResult::error(vec![Content::text(format!(
        "failed to list crates: {}",
        error
      ))])),
    }
  }

  #[tool(description = "Lookup information about a specific Rust crate")]
  fn lookup_crate(
    &self,
    Parameters(LookupCrateRequest { name }): Parameters<LookupCrateRequest>,
  ) -> Result<CallToolResult, McpError> {
    match lookup_crate(&name) {
      Ok(documentation) => match serde_yaml::to_string(&documentation) {
        Ok(yaml_content) => {
          Ok(CallToolResult::success(vec![Content::text(yaml_content)]))
        }
        Err(error) => Ok(CallToolResult::error(vec![Content::text(format!(
          "failed to serialize documentation: {}",
          error
        ))])),
      },
      Err(error) => Ok(CallToolResult::error(vec![Content::text(format!(
        "failed to lookup crate '{}': {}",
        name, error
      ))])),
    }
  }
}
