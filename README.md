# Traggo MCP

An MCP server for Traggo, a tag-based time tracking tool.

`traggo-mcp` is a standalone stdio MCP server that exposes focused tools for time tracking workflows. It talks to Traggo through the GraphQL endpoint.

## AI Disclaimer

This repo has almost entirely been **AI-generated for personal use**. It may not follow the best practices beyond those imposed by Rust and clippy.

## Configuration

Required environment variables:

- `TRAGGO_URL`: full GraphQL endpoint URL, for example `https://traggo.example.com/graphql`.
- `TRAGGO_TOKEN`: Traggo device token.

Optional environment variables:

- `TRAGGO_TIMEOUT_SECONDS`: HTTP request timeout in seconds. Defaults to `30`.
- `RUST_LOG`: tracing filter. Logs are written to stderr so stdout remains available for MCP stdio.

Create a dedicated Traggo device token in the Traggo web UI Devices tab. A `NoExpiry` device named `mcp` is convenient for trusted local deployments. Copy the token immediately because Traggo does not show existing tokens later.

## Tools

- `list_time_spans`, `create_time_span`, `update_time_span`, `remove_time_span`
- `list_timers`, `stop_timer`
- `list_tags`, `create_tag`, `update_tag`, `remove_tag`
- `suggest_tag_values`
- `get_stats`

The server validates RFC3339 timestamps, required strings, stats ranges, and pagination bounds before sending requests to Traggo.

## Native MCP Configuration

```json
{
  "mcpServers": {
    "traggo": {
      "command": "traggo-mcp",
      "env": {
        "TRAGGO_URL": "https://traggo.example.com/graphql",
        "TRAGGO_TOKEN": "..."
      }
    }
  }
}
```

## Docker MCP Configuration

The `-i` flag is required so the stdio MCP server can read from stdin.

```json
{
  "mcpServers": {
    "traggo": {
      "command": "docker",
      "args": [
        "run",
        "--rm",
        "-i",
        "-e",
        "TRAGGO_URL=https://traggo.example.com/graphql",
        "-e",
        "TRAGGO_TOKEN=...",
        "ghcr.io/IlyaBizyaev/traggo-mcp:latest"
      ]
    }
  }
}
```

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
docker build -t traggo-mcp:local .
```

## License

[GPL 3.0](LICENSE). The checked-in GraphQL schema is taken from Traggo, which itself is licensed under GPL 3.0.
