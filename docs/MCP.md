# MCP Server Setup

NeuralScope exposes observability tools via the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) over stdio. Use it from **Cursor**, **Claude Desktop**, or any MCP-compatible client.

## Available tools

All project-scoped AI tools are exposed, including:

| Tool | Description |
|------|-------------|
| `search_logs` | Search application logs |
| `search_metrics` | Query metric time-series |
| `search_traces` | Find distributed traces |
| `search_git` | Git commits and deployments |
| `search_network` | Network connection events |
| `search_architecture` | Service dependency graph |
| `search_security` | Security findings |
| `search_incidents` | Incident reports |
| `search_codebase` | Semantic code search (RAG) |
| `search_docs` | Semantic documentation search (RAG) |

## Prerequisites

1. NeuralScope stack running (`docker compose up -d`)
2. A project ID (from Settings or `GET /api/v1/projects`)
3. Database reachable from your machine

## Build the MCP binary

```bash
cd apps/server
cargo build --release --bin neuralscope-mcp
```

Binary path: `apps/server/target/release/neuralscope-mcp`

## Cursor configuration

Add to `.cursor/mcp.json` (or Cursor Settings → MCP):

```json
{
  "mcpServers": {
    "neuralscope": {
      "command": "/absolute/path/to/NeuralScope/apps/server/target/release/neuralscope-mcp",
      "env": {
        "DATABASE_URL": "postgres://neuralscope:neuralscope@localhost:5432/neuralscope",
        "REDIS_URL": "redis://localhost:6379",
        "QDRANT_URL": "http://localhost:6333",
        "NEURALSCOPE_PROJECT_ID": "your-project-uuid-here",
        "JINA_API_KEY": ""
      }
    }
  }
}
```

Replace `NEURALSCOPE_PROJECT_ID` with your project's UUID.

## Claude Desktop configuration

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "neuralscope": {
      "command": "/absolute/path/to/neuralscope-mcp",
      "env": {
        "DATABASE_URL": "postgres://neuralscope:neuralscope@localhost:5432/neuralscope",
        "REDIS_URL": "redis://localhost:6379",
        "QDRANT_URL": "http://localhost:6333",
        "NEURALSCOPE_PROJECT_ID": "your-project-uuid-here"
      }
    }
  }
}
```

## Environment variables

| Variable | Required | Description |
|----------|----------|-------------|
| `NEURALSCOPE_PROJECT_ID` | Yes | UUID of the project to query |
| `DATABASE_URL` | Yes | Postgres connection string |
| `REDIS_URL` | Yes | Redis connection string |
| `QDRANT_URL` | No | Enables RAG tools (`search_codebase`, `search_docs`) |
| `JINA_API_KEY` | No | Real embeddings; stub used if unset |

## Verify

Restart Cursor/Claude after saving config. Ask:

> "Use NeuralScope to search for recent error logs in my project."

The client should call `search_logs` via MCP.

## Security note

The MCP server connects directly to the database with the configured project scope. Run it locally or on a trusted network only. Do not expose the stdio process to untrusted users.
