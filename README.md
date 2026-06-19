# aMule MCP Server

Servidor MCP (Model Context Protocol) que permite a un modelo de IA controlar una instancia de aMule corriendo en Docker, a través del protocolo JSON-RPC 2.0 via stdin/stdout.

## Herramientas disponibles

| Herramienta        | Descripción                                      |
|--------------------|--------------------------------------------------|
| `get_status`       | Muestra el estado global de conexiones de aMule  |
| `list_downloads`   | Lista las descargas activas                      |
| `add_link`         | Añade un enlace eD2k o Magnet a la cola          |

## Requisitos

- **Rust** (edition 2021+) — [rustup.rs](https://rustup.rs)
- **Docker** con un contenedor de aMule que tenga `amulecmd` disponible
- El contenedor debe tener habilitada la interfaz remota de aMule con contraseña

## Compilación

```bash
cargo build --release
```

El binario se genera en `target/release/amule-mcp`.

## Variables de entorno

| Variable            | Requerida | Descripción                                  | Valor por defecto |
|---------------------|-----------|----------------------------------------------|--------------------|
| `AMULE_CONTAINER`   | No        | Nombre del contenedor Docker de aMule         | `amule`            |
| `AMULE_PASSWORD`    | **Sí**    | Contraseña de la interfaz remota de aMule     | —                  |

## Uso

### Ejecución directa

```bash
export AMULE_PASSWORD="tu_contraseña"
export AMULE_CONTAINER="amule"  # opcional
./target/release/amule-mcp
```

### Como servidor MCP

Configura el servidor en tu cliente MCP (por ejemplo, en `opencode`, `claude-desktop`, etc.):

```json
{
  "mcpServers": {
    "amule": {
      "command": "/ruta/a/amule-mcp",
      "env": {
        "AMULE_PASSWORD": "tu_contraseña",
        "AMULE_CONTAINER": "amule"
      }
    }
  }
}
```

### Ejemplos de uso con la IA

Una vez conectado, puedes pedirle a la IA:

- *"¿Cuál es el estado de aMule?"* → usa `get_status`
- *"Muéstrame las descargas activas"* → usa `list_downloads`
- *"Añade este enlace: ed2k://|file|..."* → usa `add_link`

## Licencia

Este proyecto está licenciado bajo la GNU General Public License v3.0 - ver el archivo [LICENSE](LICENSE) para más detalles.
