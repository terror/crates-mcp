use {
  crate::{
    parser::{list_crates, lookup_crate},
    router::Router,
  },
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
  tracing::{Level, error, info},
  tracing_subscriber::{self, EnvFilter},
};

mod parser;
mod router;

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

  info!("Starting MCP server...");

  if let Err(error) = run().await {
    error!("error: {error}");
    process::exit(1);
  }
}
