use {
  anyhow::{Error, anyhow, bail},
  arguments::Arguments,
  clap::Parser,
  documentation::Documentation,
  handler::{generate_docs, list_crates, lookup_crate},
  item::Item,
  item_kind::ItemKind,
  method::Method,
  parser::{list_crates_in_path, lookup_crate_in_path},
  regex::Regex,
  rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars::{self, JsonSchema},
    tool, tool_handler, tool_router,
    transport::io::stdio,
  },
  router::{GenerateDocsRequest, LookupCrateRequest, Router},
  scraper::{Html, Selector},
  serde::{Deserialize, Serialize},
  std::{
    fs,
    io::stderr,
    path::{Path, PathBuf},
    process::{self, Command},
  },
  subcommand::Subcommand,
  tracing::{error, info},
  tracing_subscriber::{self, EnvFilter},
};

mod arguments;
mod documentation;
mod handler;
mod item;
mod item_kind;
mod method;
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
