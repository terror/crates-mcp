use super::*;

pub async fn run() -> Result {
  info!("Starting MCP server...");
  let server = Router::new();
  let service = server.serve(stdio()).await?;
  service.waiting().await?;
  Ok(())
}
