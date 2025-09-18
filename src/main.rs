use {
  crate::parser::{list_crates, lookup_crate},
  anyhow::{Error, bail},
  regex::Regex,
  rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router,
    transport::io::stdio,
  },
  scraper::{Html, Selector},
  serde::{Deserialize, Serialize},
  std::{
    fs,
    io::stderr,
    path::{Path, PathBuf},
    process,
  },
  tracing::Level,
  tracing_subscriber::{self, EnvFilter},
};

mod parser;

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
  ) -> String {
    list_crates().unwrap().join("\n")
  }

  #[tool(description = "Lookup information about a specific Rust crate")]
  fn lookup_crate(
    &self,
    Parameters(LookupCrateRequest { name }): Parameters<LookupCrateRequest>,
  ) -> String {
    serde_yaml::to_string(&lookup_crate(&name).unwrap()).unwrap()
  }
}

async fn run() -> Result {
  let server = Router::new();
  let service = server.serve(stdio()).await?;
  service.waiting().await?;
  Ok(())
}

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .with_env_filter(
      EnvFilter::from_default_env().add_directive(Level::DEBUG.into()),
    )
    .with_writer(stderr)
    .with_ansi(false)
    .init();

  if let Err(error) = run().await {
    eprintln!("error: {error}");
    process::exit(1);
  }
}
