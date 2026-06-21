# aMule MCP Server

MCP (Model Context Protocol) server that allows an AI model to control an aMule instance running in Docker, through the JSON-RPC 2.0 protocol via stdin/stdout.

## Available tools

| Tool                   | Description                                           |
|------------------------|-------------------------------------------------------|
| `get_status`           | Shows aMule's global connection status                |
| `list_downloads`       | Lists active downloads                                |
| `add_link`             | Adds an eD2k or Magnet link to the queue              |
| `pause_download`       | Pauses a single download by hash                      |
| `pause_all_downloads`  | Pauses all active downloads                           |
| `resume_download`      | Resumes a paused download by hash                     |
| `resume_all_downloads` | Resumes all paused downloads                          |
| `cancel_download`      | Removes/cancels a single download by hash             |
| `cancel_all_downloads` | Removes all active downloads                          |
| `set_priority`         | Changes priority of a single download (Low/Normal/High/Auto) |
| `set_priority_all`     | Changes priority of all downloads                     |

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
- *"Pause download ABC123"* → uses `pause_download`
- *"Pause all downloads"* → uses `pause_all_downloads`
- *"Resume all paused downloads"* → uses `resume_all_downloads`
- *"Cancel download ABC123"* → uses `cancel_download`
- *"Set priority High for download ABC123"* → uses `set_priority`
- *"Set all downloads to Low priority"* → uses `set_priority_all`

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
