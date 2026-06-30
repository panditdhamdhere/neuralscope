//! NeuralScope MCP server — exposes observability tools over stdio for Cursor and Claude Desktop.

use std::sync::Arc;

use neuralscope_server::{
    db, mcp, vector,
    AppConfig,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()?;
    let bundle = db::connect(&config).await?;

    let embedding_provider = vector::infrastructure::create_embedding_provider(&config);
    let vector_service = Arc::new(vector::application::VectorService::from_parts(
        embedding_provider,
        &config.qdrant_url,
        bundle.pool.clone(),
    ));

    mcp::run_stdio_server(bundle.pool, Some(vector_service)).await
}
