// aMule MCP Server - Control aMule via MCP protocol
// Copyright (C) 2026
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use serde_json::{json, Value};

// Ejecuta el comando en Docker y devuelve el texto
async fn run_amule_cmd(container: &str, password: &str, action: &str) -> String {
    let output = Command::new("docker")
        .args(&[
            "exec", "-i", container, 
            "amulecmd", "-P", password, "-c", action
        ])
        .output()
        .await;

    match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        Ok(out) => format!("Error de amulecmd: {}", String::from_utf8_lossy(&out.stderr)),
        Err(e) => format!("Error al ejecutar docker: {}", e),
    }
}

#[tokio::main]
async fn main() {
    let container = env::var("AMULE_CONTAINER").unwrap_or_else(|_| "amule".to_string());
    let password = env::var("AMULE_PASSWORD").expect("CRÍTICO: Variable AMULE_PASSWORD no definida");

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin).lines();

    // Bucle infinito leyendo lo que pide la IA
    while let Ok(Some(line)) = reader.next_line().await {
        let req: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(id) = req.get("id") {
            let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

            let response = match method {
                // 1. Inicialización
                "initialize" => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": { "tools": {} },
                        "serverInfo": { "name": "amule-mcp", "version": "0.1.0" }
                    }
                }),
                // 2. Lista de herramientas disponibles
                "tools/list" => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "tools": [
                            {
                                "name": "get_status",
                                "description": "Muestra el estado global de conexiones de aMule",
                                "inputSchema": { "type": "object", "properties": {} }
                            },
                            {
                                "name": "list_downloads",
                                "description": "Muestra la lista de descargas activas",
                                "inputSchema": { "type": "object", "properties": {} }
                            },
                            {
                                "name": "add_link",
                                "description": "Añade un nuevo enlace eD2k o Magnet",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "link": { "type": "string", "description": "El enlace eD2k completo" }
                                    },
                                    "required": ["link"]
                                }
                            }
                        ]
                    }
                }),
                // 3. Ejecución de herramientas
                "tools/call" => {
                    let name = req.get("params")
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("");

                    let result_text = match name {
                        "get_status" => run_amule_cmd(&container, &password, "status").await,
                        "list_downloads" => run_amule_cmd(&container, &password, "show DL").await,
                        "add_link" => {
                            let link = req.get("params")
                                .and_then(|p| p.get("arguments"))
                                .and_then(|a| a.get("link"))
                                .and_then(|l| l.as_str())
                                .unwrap_or("");

                            if link.starts_with("ed2k://") || link.starts_with("magnet:") {
                                run_amule_cmd(&container, &password, &format!("Add {}", link)).await
                            } else {
                                "Error: El enlace no tiene un formato válido".to_string()
                            }
                        },
                        _ => format!("Herramienta desconocida: {}", name)
                    };

                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "content": [ { "type": "text", "text": result_text } ]
                        }
                    })
                },
                // 4. Fallback para métodos desconocidos
                _ => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32601, "message": "Method not found" }
                }),
            };

            // Enviamos la respuesta a Hermes
            let mut res_str = serde_json::to_string(&response).unwrap();
            res_str.push('\n');
            let _ = stdout.write_all(res_str.as_bytes()).await;
            let _ = stdout.flush().await;
        }
    }
}
