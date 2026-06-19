# aMule MCP Server

MCP (Model Context Protocol) server that allows an AI model to control an aMule instance running in Docker, through the JSON-RPC 2.0 protocol via stdin/stdout.

## Available tools

| Tool               | Description                                      |
|--------------------|--------------------------------------------------|
| `get_status`       | Shows aMule's global connection status           |
| `list_downloads`   | Lists active downloads                           |
| `add_link`         | Adds an eD2k or Magnet link to the queue         |

## Requirements

- **Rust** (edition 2021+) — [rustup.rs](https://rustup.rs)
- **Docker** with an aMule container that has `amulecmd` available
- The container must have aMule's remote interface enabled with a password

## Compilation

```bash
cargo build --release
```

The binary is generated at `target/release/amule-mcp`.

## Environment variables

| Variable            | Required | Description                                  | Default value |
|---------------------|----------|----------------------------------------------|----------------|
| `AMULE_CONTAINER`   | No       | Name of the aMule Docker container           | `amule`        |
| `AMULE_PASSWORD`    | **Yes**  | Password for aMule's remote interface        | —              |

## Usage

### Direct execution

```bash
export AMULE_PASSWORD="your_password"
export AMULE_CONTAINER="amule"  # optional
./target/release/amule-mcp
```

### As an MCP server

Install
`cargo install --quiet amule-mcp `


Configure the server in your MCP client (for example, in `opencode`, `claude-desktop`, etc.):

```json
{
  "mcpServers": {
    "amule": {
      "command": "/path/to/amule-mcp",
      "env": {
        "AMULE_PASSWORD": "your_password",
        "AMULE_CONTAINER": "amule"
      }
    }
  }
}
```

### AI usage examples

Once connected, you can ask the AI:

- *"What is the status of aMule?"* → uses `get_status`
- *"Show me active downloads"* → uses `list_downloads`
- *"Add this link: ed2k://|file|..."* → uses `add_link`

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
