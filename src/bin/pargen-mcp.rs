//! MCP server binary for pargen
//!
//! This binary starts an MCP server that exposes pargen functionality
//! via the Model Context Protocol.
//!
//! Usage:
//!   pargen-mcp

use pargen::mcp::create_server;
use rmcp::service::serve_server;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = create_server();
    serve_server(server, stdio()).await?;
    Ok(())
}
