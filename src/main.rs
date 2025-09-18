use {
  anyhow::{Error, anyhow, bail},
  arguments::Arguments,
  clap::Parser,
  parser::{list_crates, lookup_crate_with_options},
  regex::Regex,
  rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router,
    transport::io::stdio,
  },
  router::Router,
  scraper::{Html, Selector},
  serde::{Deserialize, Serialize},
  std::{
    fs,
    io::stderr,
    path::{Path, PathBuf},
    process,
  },
  subcommand::Subcommand,
  tracing::{error, info},
  tracing_subscriber::{self, EnvFilter},
};

mod arguments;
mod parser;
mod router;
mod subcommand;

type Result<T = (), E = Error> = std::result::Result<T, E>;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .with_writer(stderr)
    .with_ansi(false)
    .init();

  if let Err(error) = Arguments::parse().run().await {
    error!("error: {error}");
    process::exit(1);
  }
}
