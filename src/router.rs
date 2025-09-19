use super::*;

const DOC_PATH: &str = "target/doc";

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateDocsRequest {
  #[schemars(
    description = "Additional cargo doc flags (e.g., '--no-deps', '--open')"
  )]
  pub flags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListCratesRequest {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LookupCrateRequest {
  #[schemars(description = "The name of the Rust crate")]
  pub name: String,
  #[schemars(
    description = "Maximum number of items to return (default: no limit)"
  )]
  pub limit: Option<usize>,
  #[schemars(
    description = "Number of items to skip for pagination (default: 0)"
  )]
  pub offset: Option<usize>,
  #[schemars(
    description = "Filter by item type: function, struct, enum, trait, macro, type, constant, module"
  )]
  pub item_type: Option<String>,
  #[schemars(
    description = "Search term to filter items by name or description"
  )]
  pub query: Option<String>,
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

  #[tool(description = "Generate documentation using 'cargo doc'")]
  fn generate_docs(
    &self,
    Parameters(parameters): Parameters<GenerateDocsRequest>,
  ) -> Result<CallToolResult, McpError> {
    match self.generate_docs_impl(&parameters) {
      Ok(result) => Ok(CallToolResult::success(vec![Content::text(result)])),
      Err(error) => Err(error.into()),
    }
  }

  fn generate_docs_impl(
    &self,
    parameters: &GenerateDocsRequest,
  ) -> Result<String, Error> {
    let output = Command::new("cargo")
      .arg("doc")
      .args(parameters.flags.as_deref().unwrap_or(&[]))
      .output()
      .map_err(|error| anyhow!("failed to run cargo doc: {}", error))?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
      return Err(anyhow!("cargo doc failed: {}", stderr).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    Ok(format!("{}{}", stdout, stderr))
  }

  #[tool(description = "List all available Rust crates")]
  fn list_crates(
    &self,
    Parameters(ListCratesRequest {}): Parameters<ListCratesRequest>,
  ) -> Result<CallToolResult, McpError> {
    match list_crates(DOC_PATH) {
      Ok(crates) => Ok(CallToolResult::success(vec![Content::text(
        crates.join("\n"),
      )])),
      Err(error) => Err(error.into()),
    }
  }

  #[tool(description = "Lookup information about a specific Rust crate")]
  fn lookup_crate(
    &self,
    Parameters(parameters): Parameters<LookupCrateRequest>,
  ) -> Result<CallToolResult, McpError> {
    match self.lookup_crate_impl(&parameters) {
      Ok(content) => Ok(CallToolResult::success(vec![Content::text(content)])),
      Err(error) => Err(error.into()),
    }
  }

  fn lookup_crate_impl(
    &self,
    parameters: &LookupCrateRequest,
  ) -> Result<String> {
    Ok(serde_json::to_string(&lookup_crate(parameters, DOC_PATH)?)?)
  }
}
