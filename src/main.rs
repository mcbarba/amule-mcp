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

fn is_valid_link(link: &str) -> bool {
    link.starts_with("ed2k://") || link.starts_with("magnet:")
}

fn extract_tool_name(req: &Value) -> &str {
    req.get("params")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("")
}

fn extract_link(req: &Value) -> &str {
    req.get("params")
        .and_then(|p| p.get("arguments"))
        .and_then(|a| a.get("link"))
        .and_then(|l| l.as_str())
        .unwrap_or("")
}

fn extract_hash(req: &Value) -> &str {
    req.get("params")
        .and_then(|p| p.get("arguments"))
        .and_then(|a| a.get("hash"))
        .and_then(|h| h.as_str())
        .unwrap_or("")
}

fn extract_priority(req: &Value) -> &str {
    req.get("params")
        .and_then(|p| p.get("arguments"))
        .and_then(|a| a.get("priority"))
        .and_then(|p| p.as_str())
        .unwrap_or("")
}

fn is_valid_priority(priority: &str) -> bool {
    matches!(priority.to_lowercase().as_str(), "low" | "normal" | "high" | "auto")
}

fn normalize_priority(priority: &str) -> &str {
    match priority.to_lowercase().as_str() {
        "low" => "Low",
        "normal" => "Normal",
        "high" => "High",
        "auto" => "Auto",
        _ => priority,
    }
}

fn build_error_response(id: &Value, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}

fn build_success_response(id: &Value, result_text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [ { "type": "text", "text": result_text } ]
        }
    })
}

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
                            },
                            {
                                "name": "pause_download",
                                "description": "Pausa una descarga individual por su hash",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "hash": { "type": "string", "description": "Hash de la descarga" }
                                    },
                                    "required": ["hash"]
                                }
                            },
                            {
                                "name": "pause_all_downloads",
                                "description": "Pausa todas las descargas activas",
                                "inputSchema": { "type": "object", "properties": {} }
                            },
                            {
                                "name": "resume_download",
                                "description": "Reanuda una descarga pausada por su hash",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "hash": { "type": "string", "description": "Hash de la descarga" }
                                    },
                                    "required": ["hash"]
                                }
                            },
                            {
                                "name": "resume_all_downloads",
                                "description": "Reanuda todas las descargas pausadas",
                                "inputSchema": { "type": "object", "properties": {} }
                            },
                            {
                                "name": "cancel_download",
                                "description": "Elimina/cancela una descarga individual por su hash",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "hash": { "type": "string", "description": "Hash de la descarga" }
                                    },
                                    "required": ["hash"]
                                }
                            },
                            {
                                "name": "cancel_all_downloads",
                                "description": "Elimina todas las descargas activas",
                                "inputSchema": { "type": "object", "properties": {} }
                            },
                            {
                                "name": "set_priority",
                                "description": "Cambia la prioridad de una descarga individual",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "hash": { "type": "string", "description": "Hash de la descarga" },
                                        "priority": { "type": "string", "enum": ["Low", "Normal", "High", "Auto"], "description": "Nueva prioridad" }
                                    },
                                    "required": ["hash", "priority"]
                                }
                            },
                            {
                                "name": "set_priority_all",
                                "description": "Cambia la prioridad de todas las descargas",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "priority": { "type": "string", "enum": ["Low", "Normal", "High", "Auto"], "description": "Nueva prioridad" }
                                    },
                                    "required": ["priority"]
                                }
                            }
                        ]
                    }
                }),
                // 3. Ejecución de herramientas
                "tools/call" => {
                    let name = extract_tool_name(&req);

                    let result_text = match name {
                        "get_status" => run_amule_cmd(&container, &password, "status").await,
                        "list_downloads" => run_amule_cmd(&container, &password, "show DL").await,
                        "add_link" => {
                            let link = extract_link(&req);
                            if is_valid_link(link) {
                                run_amule_cmd(&container, &password, &format!("Add {}", link)).await
                            } else {
                                "Error: El enlace no tiene un formato válido".to_string()
                            }
                        },
                        "pause_download" => {
                            let hash = extract_hash(&req);
                            if hash.is_empty() {
                                "Error: Hash requerido".to_string()
                            } else {
                                run_amule_cmd(&container, &password, &format!("Pause {}", hash)).await
                            }
                        },
                        "pause_all_downloads" => {
                            run_amule_cmd(&container, &password, "Pause all").await
                        },
                        "resume_download" => {
                            let hash = extract_hash(&req);
                            if hash.is_empty() {
                                "Error: Hash requerido".to_string()
                            } else {
                                run_amule_cmd(&container, &password, &format!("Resume {}", hash)).await
                            }
                        },
                        "resume_all_downloads" => {
                            run_amule_cmd(&container, &password, "Resume all").await
                        },
                        "cancel_download" => {
                            let hash = extract_hash(&req);
                            if hash.is_empty() {
                                "Error: Hash requerido".to_string()
                            } else {
                                run_amule_cmd(&container, &password, &format!("Cancel {}", hash)).await
                            }
                        },
                        "cancel_all_downloads" => {
                            run_amule_cmd(&container, &password, "Cancel all").await
                        },
                        "set_priority" => {
                            let hash = extract_hash(&req);
                            let priority = extract_priority(&req);
                            if hash.is_empty() {
                                "Error: Hash requerido".to_string()
                            } else if !is_valid_priority(priority) {
                                "Error: Prioridad inválida. Usa: Low, Normal, High o Auto".to_string()
                            } else {
                                let prio = normalize_priority(priority);
                                run_amule_cmd(&container, &password, &format!("Priority {} {}", hash, prio)).await
                            }
                        },
                        "set_priority_all" => {
                            let priority = extract_priority(&req);
                            if !is_valid_priority(priority) {
                                "Error: Prioridad inválida. Usa: Low, Normal, High o Auto".to_string()
                            } else {
                                let prio = normalize_priority(priority);
                                run_amule_cmd(&container, &password, &format!("Priority all {}", prio)).await
                            }
                        },
                        _ => format!("Herramienta desconocida: {}", name)
                    };

                    build_success_response(id, &result_text)
                },
                _ => build_error_response(id, -32601, "Method not found"),
            };

            // Enviamos la respuesta a Hermes
            let mut res_str = serde_json::to_string(&response).unwrap();
            res_str.push('\n');
            let _ = stdout.write_all(res_str.as_bytes()).await;
            let _ = stdout.flush().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_valid_link_ed2k() {
        assert!(is_valid_link("ed2k://|file|test.avi|123456|hash|/"));
    }

    #[test]
    fn test_is_valid_link_magnet() {
        assert!(is_valid_link("magnet:?xt=urn:btih:abc123"));
    }

    #[test]
    fn test_is_valid_link_invalid() {
        assert!(!is_valid_link("http://example.com"));
        assert!(!is_valid_link("ftp://files.com"));
        assert!(!is_valid_link(""));
        assert!(!is_valid_link("ed2k"));
    }

    #[test]
    fn test_extract_tool_name() {
        let req = json!({
            "params": {
                "name": "get_status",
                "arguments": {}
            }
        });
        assert_eq!(extract_tool_name(&req), "get_status");
    }

    #[test]
    fn test_extract_tool_name_missing() {
        let req = json!({});
        assert_eq!(extract_tool_name(&req), "");
    }

    #[test]
    fn test_extract_tool_name_no_params() {
        let req = json!({ "method": "tools/call" });
        assert_eq!(extract_tool_name(&req), "");
    }

    #[test]
    fn test_extract_link() {
        let req = json!({
            "params": {
                "name": "add_link",
                "arguments": {
                    "link": "ed2k://|file|test.avi|123|hash|/"
                }
            }
        });
        assert_eq!(extract_link(&req), "ed2k://|file|test.avi|123|hash|/");
    }

    #[test]
    fn test_extract_link_missing() {
        let req = json!({
            "params": {
                "name": "add_link",
                "arguments": {}
            }
        });
        assert_eq!(extract_link(&req), "");
    }

    #[test]
    fn test_extract_link_no_arguments() {
        let req = json!({ "params": { "name": "add_link" } });
        assert_eq!(extract_link(&req), "");
    }

    #[test]
    fn test_build_error_response() {
        let id = json!(1);
        let response = build_error_response(&id, -32601, "Method not found");
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);
        assert_eq!(response["error"]["code"], -32601);
        assert_eq!(response["error"]["message"], "Method not found");
    }

    #[test]
    fn test_build_error_response_string_id() {
        let id = json!("abc-123");
        let response = build_error_response(&id, -32600, "Invalid request");
        
        assert_eq!(response["id"], "abc-123");
        assert_eq!(response["error"]["code"], -32600);
    }

    #[test]
    fn test_build_success_response() {
        let id = json!(42);
        let response = build_success_response(&id, "OK: Connected");
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 42);
        assert_eq!(response["result"]["content"][0]["type"], "text");
        assert_eq!(response["result"]["content"][0]["text"], "OK: Connected");
    }

    #[test]
    fn test_build_success_response_multiline() {
        let id = json!(1);
        let text = "Line 1\nLine 2\nLine 3";
        let response = build_success_response(&id, text);
        
        assert_eq!(response["result"]["content"][0]["text"], text);
    }

    #[test]
    fn test_build_success_response_empty() {
        let id = json!(1);
        let response = build_success_response(&id, "");
        
        assert_eq!(response["result"]["content"][0]["text"], "");
    }

    #[test]
    fn test_extract_hash() {
        let req = json!({
            "params": {
                "name": "pause_download",
                "arguments": {
                    "hash": "ABC123DEF456"
                }
            }
        });
        assert_eq!(extract_hash(&req), "ABC123DEF456");
    }

    #[test]
    fn test_extract_hash_missing() {
        let req = json!({
            "params": {
                "name": "pause_download",
                "arguments": {}
            }
        });
        assert_eq!(extract_hash(&req), "");
    }

    #[test]
    fn test_extract_hash_no_arguments() {
        let req = json!({ "params": { "name": "pause_download" } });
        assert_eq!(extract_hash(&req), "");
    }

    #[test]
    fn test_extract_priority() {
        let req = json!({
            "params": {
                "name": "set_priority",
                "arguments": {
                    "hash": "ABC123",
                    "priority": "High"
                }
            }
        });
        assert_eq!(extract_priority(&req), "High");
    }

    #[test]
    fn test_extract_priority_missing() {
        let req = json!({
            "params": {
                "name": "set_priority",
                "arguments": {
                    "hash": "ABC123"
                }
            }
        });
        assert_eq!(extract_priority(&req), "");
    }

    #[test]
    fn test_is_valid_priority_low() {
        assert!(is_valid_priority("Low"));
        assert!(is_valid_priority("low"));
        assert!(is_valid_priority("LOW"));
    }

    #[test]
    fn test_is_valid_priority_normal() {
        assert!(is_valid_priority("Normal"));
        assert!(is_valid_priority("normal"));
    }

    #[test]
    fn test_is_valid_priority_high() {
        assert!(is_valid_priority("High"));
        assert!(is_valid_priority("high"));
    }

    #[test]
    fn test_is_valid_priority_auto() {
        assert!(is_valid_priority("Auto"));
        assert!(is_valid_priority("auto"));
    }

    #[test]
    fn test_is_valid_priority_invalid() {
        assert!(!is_valid_priority("urgent"));
        assert!(!is_valid_priority(""));
        assert!(!is_valid_priority("medium"));
    }

    #[test]
    fn test_normalize_priority() {
        assert_eq!(normalize_priority("low"), "Low");
        assert_eq!(normalize_priority("LOW"), "Low");
        assert_eq!(normalize_priority("normal"), "Normal");
        assert_eq!(normalize_priority("HIGH"), "High");
        assert_eq!(normalize_priority("auto"), "Auto");
    }
}
