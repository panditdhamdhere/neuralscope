//! Stdio MCP server exposing NeuralScope observability tools.

use std::io::{self, BufRead, Write};
use std::sync::Arc;

use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::ai::application::ToolRegistry;
use crate::mcp::protocol::{
    InitializeCapabilities, InitializeResult, JsonRpcRequest, JsonRpcResponse, McpTool,
    ServerInfo, ToolCallResult, ToolCapability, ToolContent, ToolListResult,
};
use crate::vector::application::VectorService;

/// Runs the MCP server on stdin/stdout until EOF.
pub async fn run_stdio_server(pool: PgPool, vector: Option<Arc<VectorService>>) -> anyhow::Result<()> {
    let project_id = std::env::var("NEURALSCOPE_PROJECT_ID")
        .map_err(|_| anyhow::anyhow!("NEURALSCOPE_PROJECT_ID is required"))?
        .parse::<Uuid>()
        .map_err(|_| anyhow::anyhow!("NEURALSCOPE_PROJECT_ID must be a valid UUID"))?;

    let registry = ToolRegistry::for_project(project_id, pool, vector);
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(request) => request,
            Err(error) => {
                let response = JsonRpcResponse::error(
                    Value::Null,
                    -32700,
                    format!("Parse error: {error}"),
                );
                write_response(&mut stdout, &response)?;
                continue;
            }
        };

        let id = request.id.clone().unwrap_or(Value::Null);
        let response = handle_request(&registry, &request.method, request.params.as_ref(), id)
            .await;
        write_response(&mut stdout, &response)?;
    }

    Ok(())
}

fn write_response(stdout: &mut io::Stdout, response: &JsonRpcResponse) -> anyhow::Result<()> {
    let payload = serde_json::to_string(response)?;
    writeln!(stdout, "{payload}")?;
    stdout.flush()?;
    Ok(())
}

async fn handle_request(
    registry: &ToolRegistry,
    method: &str,
    params: Option<&Value>,
    id: Value,
) -> JsonRpcResponse {
    match method {
        "initialize" => JsonRpcResponse::success(
            id,
            serde_json::to_value(InitializeResult {
                protocol_version: "2024-11-05",
                capabilities: InitializeCapabilities {
                    tools: ToolCapability {
                        list_changed: false,
                    },
                },
                server_info: ServerInfo {
                    name: "neuralscope",
                    version: crate::VERSION,
                },
            })
            .expect("serialize initialize"),
        ),
        "notifications/initialized" | "initialized" => JsonRpcResponse::success(id, json!({})),
        "ping" => JsonRpcResponse::success(id, json!({})),
        "tools/list" => {
            let tools: Vec<McpTool> = registry
                .definitions()
                .into_iter()
                .map(|definition| McpTool {
                    name: definition.name,
                    description: definition.description,
                    input_schema: definition.parameters,
                })
                .collect();

            JsonRpcResponse::success(
                id,
                serde_json::to_value(ToolListResult { tools }).expect("serialize tools"),
            )
        }
        "tools/call" => {
            let Some(params) = params else {
                return JsonRpcResponse::error(id, -32602, "Missing params");
            };

            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            match registry.execute(name, arguments).await {
                Ok(result) => JsonRpcResponse::success(
                    id,
                    serde_json::to_value(ToolCallResult {
                        content: vec![ToolContent {
                            content_type: "text",
                            text: result,
                        }],
                        is_error: false,
                    })
                    .expect("serialize tool result"),
                ),
                Err(error) => JsonRpcResponse::success(
                    id,
                    serde_json::to_value(ToolCallResult {
                        content: vec![ToolContent {
                            content_type: "text",
                            text: error.to_string(),
                        }],
                        is_error: true,
                    })
                    .expect("serialize tool error"),
                ),
            }
        }
        other => JsonRpcResponse::error(id, -32601, format!("Method not found: {other}")),
    }
}
