//! MCP (Model Context Protocol) server types and stdio transport.

mod protocol;
mod server;

pub use server::run_stdio_server;
