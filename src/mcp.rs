//! `umlcraft mcp`: a Model Context Protocol server on stdin and stdout (JSON-RPC 2.0, one message
//! per line, https://modelcontextprotocol.io/specification), so that AI tools such as Claude Desktop
//! or Cursor can draw and read diagrams.
//!
//! It offers the tools of the StarUML MCP server (github.com/staruml/staruml-mcp-server, src/index.ts):
//! `generate_diagram`, `get_all_diagrams_info`, `get_current_diagram_info` and
//! `get_diagram_image_by_id`, with the same descriptions and answers. Like that server, by default it
//! forwards each call to the local HTTP API of the running window (src/api.rs), on port 58321 or the
//! one given by `--api-port=<port>`.
//!
//! UMLCraft adds a second mode: `umlcraft mcp <file.mdj>` works on that file without a window. The
//! same tools answer from the file, a generated diagram is saved into it, and one more tool,
//! `execute_command`, runs the commands of `umlcraft exec` (src/exec.rs) and saves.

use crate::app::App;
use crate::canvas::EguiMeasure;
use serde_json::{json, Value};
use std::io::{BufRead, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;

/// The protocol versions this server speaks, newest first (it answers an `initialize` with the
/// client's version when it is one of them, else with the newest).
const PROTOCOL_VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

/// Where the tool calls go.
pub enum Backend {
    /// The running window's HTTP API on this port.
    Api(u16),
    /// A file opened here, without a window.
    File { app: Box<App>, path: PathBuf, ms: EguiMeasure },
}

/// The tools: name, description and input schema (index.ts:38-120).
fn tools(file_mode: bool) -> Value {
    let mut list = vec![
        json!({"name": "generate_diagram", "description": "Generate a diagram in UMLCraft.", "inputSchema": {"type": "object", "properties": {"code": {"type": "string", "description": "Mermaid code to generate the diagram. Supported diagrams are classDiagram, sequenceDiagram, flowchart, erDiagram, mindmap, requirementDiagram and stateDiagram. Other diagrams are not supported."}}, "required": ["code"]}}),
        json!({"name": "get_all_diagrams_info", "description": "Get information for all diagrams in UMLCraft.", "inputSchema": {"type": "object", "properties": {}}}),
        json!({"name": "get_current_diagram_info", "description": "Get information for the current diagram in UMLCraft.", "inputSchema": {"type": "object", "properties": {}}}),
        json!({"name": "get_diagram_image_by_id", "description": "Get the image of a diagram by its ID in UMLCraft.", "inputSchema": {"type": "object", "properties": {"diagramId": {"type": "string", "description": "ID of the diagram to get the image for. You can get the ID from the 'get_all_diagrams_info' tool."}}, "required": ["diagramId"]}}),
    ];
    if file_mode {
        let ids: Vec<&str> = crate::exec::COMMANDS.iter().map(|c| c.0).collect();
        list.push(json!({"name": "execute_command", "description": format!("Run a UMLCraft command on the model and save the file. The commands are: {}. Factory commands (factory:create-diagram, factory:create-model, factory:create-model-and-view) and engine:set-property take a JSON object as their argument, as `umlcraft exec -a` does.", ids.join(", ")), "inputSchema": {"type": "object", "properties": {"command": {"type": "string", "description": "The command id, such as factory:create-model-and-view."}, "arg": {"type": "string", "description": "The command's argument: a string, or JSON text for the factory commands."}}, "required": ["command"]}}));
    }
    Value::Array(list)
}

fn text(t: impl Into<String>) -> Value {
    json!({"content": [{"type": "text", "text": t.into()}]})
}

/// A failed tool call: the error is part of the result, so the model can read it (response.ts:32-52).
fn tool_error(t: impl Into<String>) -> Value {
    json!({"isError": true, "content": [{"type": "text", "text": t.into()}]})
}

/// POST a JSON body to the API and return its `data` (utils.ts:89-117).
fn api_post(port: u16, slug: &str, args: &Value) -> Result<Value, String> {
    let body = args.to_string();
    let mut s = TcpStream::connect(("127.0.0.1", port)).map_err(|e| format!("Cannot reach UMLCraft on port {port} (is it running with its API server on?): {e}"))?;
    s.set_read_timeout(Some(Duration::from_secs(90))).ok();
    write!(s, "POST {slug} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).map_err(|e| e.to_string())?;
    let mut raw = vec![];
    s.read_to_end(&mut raw).map_err(|e| e.to_string())?;
    let raw = String::from_utf8_lossy(&raw);
    let (head, payload) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
    let status: u16 = head.split_whitespace().nth(1).and_then(|c| c.parse().ok()).unwrap_or(0);
    let json: Value = serde_json::from_str(payload).unwrap_or(Value::Null);
    let error = json.get("error").and_then(|e| e.as_str());
    if !(200..300).contains(&status) {
        return Err(format!("Failed to execute command({slug}) with args: {args}{}", error.map(|e| format!(" ({e})")).unwrap_or_default()));
    }
    if json.get("success").and_then(|s| s.as_bool()) != Some(true) {
        return Err(format!("Failed to call API ({slug}): {}", error.unwrap_or("undefined")));
    }
    Ok(json.get("data").cloned().unwrap_or(Value::Null))
}

impl Backend {
    /// Call one API path, through the window or on the open file.
    fn call(&mut self, slug: &str, args: &Value) -> Result<Value, String> {
        match self {
            Backend::Api(port) => api_post(*port, slug, args),
            Backend::File { app, path, ms } => {
                let (status, answer) = app.api_answer(slug, args, &*ms);
                let json: Value = serde_json::from_str(&answer).unwrap_or(Value::Null);
                let error = json.get("error").and_then(|e| e.as_str()).unwrap_or("undefined");
                if status != 200 {
                    return Err(format!("Failed to execute command({slug}) with args: {args} ({error})"));
                }
                if slug == "/generate_diagram" && !app.save_to(path.clone(), &*ms) {
                    return Err(format!("Cannot save {}", path.display()));
                }
                Ok(json.get("data").cloned().unwrap_or(Value::Null))
            }
        }
    }

    /// Run a tool (index.ts:38-120); `None` for an unknown tool.
    fn tool(&mut self, name: &str, args: &Value) -> Option<Value> {
        let string = |k: &str| args.get(k).and_then(|v| v.as_str()).map(str::to_string);
        Some(match name {
            "generate_diagram" => {
                let Some(code) = string("code") else { return Some(tool_error("Invalid arguments: code must be a string")) };
                match self.call("/generate_diagram", &json!({"code": code})) {
                    Ok(_) => text("Diagram is successfully generated."),
                    Err(e) => tool_error(format!("Failed to generate diagram: {e}")),
                }
            }
            "get_all_diagrams_info" => match self.call("/get_all_diagrams_info", &json!({})) {
                Ok(d) => text(format!("All diagrams: {d}")),
                Err(e) => tool_error(format!("Failed to get all diagrams info: {e}")),
            },
            "get_current_diagram_info" => match self.call("/get_current_diagram_info", &json!({})) {
                Ok(Value::Null) => text("No current diagram found."),
                Ok(d) => text(format!("Current diagram: {d}")),
                Err(e) => tool_error(format!("Failed to get current diagram info: {e}")),
            },
            "get_diagram_image_by_id" => {
                let Some(id) = string("diagramId") else { return Some(tool_error("Invalid arguments: diagramId must be a string")) };
                match self.call("/get_diagram_image_by_id", &json!({"diagramId": id})) {
                    Ok(Value::String(png)) => json!({"content": [{"type": "image", "data": png, "mimeType": "image/png"}]}),
                    // an empty diagram has no picture; the API answers null
                    Ok(_) => text("The diagram is empty, so it has no image."),
                    Err(e) => tool_error(format!("Failed to get diagram image by id: {e}")),
                }
            }
            "execute_command" => {
                let Backend::File { app, path, ms } = self else { return None };
                let Some(cmd) = string("command") else { return Some(tool_error("Invalid arguments: command must be a string")) };
                if !crate::exec::COMMANDS.iter().any(|c| c.0 == cmd) {
                    return Some(tool_error(format!("Command not found: {cmd}")));
                }
                match app.exec_command(&cmd, string("arg").as_deref(), &[], &*ms) {
                    Ok(made) => {
                        if app.modified && !app.save_to(path.clone(), &*ms) {
                            return Some(tool_error(format!("Cannot save {}", path.display())));
                        }
                        match made {
                            Some(e) => text(format!("Done: {} {}", app.repo.ty(e), app.repo.id(e))),
                            None => text("Done."),
                        }
                    }
                    Err(e) => tool_error(format!("Failed to execute command {cmd}: {e}")),
                }
            }
            _ => return None,
        })
    }

    /// Answer one JSON-RPC message; `None` for a notification, which gets no answer.
    pub fn handle(&mut self, msg: &Value) -> Option<Value> {
        let id = msg.get("id").cloned();
        let Some(method) = msg.get("method").and_then(|m| m.as_str()) else {
            // a response to a request of ours (we send none), or not a request at all
            return id.filter(|_| msg.get("result").is_none() && msg.get("error").is_none()).map(|id| error(id, -32600, "Invalid Request"));
        };
        let id = id?;
        let params = msg.get("params").cloned().unwrap_or(json!({}));
        let result = match method {
            "initialize" => {
                let asked = params.get("protocolVersion").and_then(|v| v.as_str()).unwrap_or("");
                let version = PROTOCOL_VERSIONS.iter().find(|v| **v == asked).unwrap_or(&PROTOCOL_VERSIONS[0]);
                json!({"protocolVersion": version, "capabilities": {"tools": {}}, "serverInfo": {"name": "umlcraft", "version": env!("CARGO_PKG_VERSION")}})
            }
            "ping" => json!({}),
            "tools/list" => json!({"tools": tools(matches!(self, Backend::File { .. }))}),
            "tools/call" => {
                let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                match self.tool(name, &args) {
                    Some(r) => r,
                    None => return Some(error(id, -32602, &format!("Unknown tool: {name}"))),
                }
            }
            _ => return Some(error(id, -32601, &format!("Method not found: {method}"))),
        };
        Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
    }
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

/// Serve on stdin and stdout until stdin closes.
pub fn serve(mut backend: Backend, input: impl BufRead, mut output: impl Write) -> std::io::Result<()> {
    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let answer = match serde_json::from_str::<Value>(&line) {
            Err(_) => Some(error(Value::Null, -32700, "Parse error")),
            // a batch (protocol 2025-03-26): the answers in one array
            Ok(Value::Array(msgs)) => {
                let all: Vec<Value> = msgs.iter().filter_map(|m| backend.handle(m)).collect();
                (!all.is_empty()).then_some(Value::Array(all))
            }
            Ok(msg) => backend.handle(&msg),
        };
        if let Some(a) = answer {
            writeln!(output, "{a}")?;
            output.flush()?;
        }
    }
    Ok(())
}

/// `umlcraft mcp [file.mdj] [--api-port=<port>]`: returns the exit code.
pub fn cli(args: &[String], ctx: impl Fn() -> egui::Context) -> i32 {
    let mut port = crate::api::DEFAULT_PORT;
    let mut file = None;
    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        let value = if let Some(v) = a.strip_prefix("--api-port=") {
            Some(v.to_string())
        } else if a == "--api-port" {
            i += 1;
            args.get(i).cloned()
        } else {
            if !a.starts_with('-') {
                file = Some(PathBuf::from(a));
            }
            None
        };
        if let Some(v) = value {
            // index.ts:16-30
            match v.parse::<u16>() {
                Ok(p) => port = p,
                Err(_) => {
                    eprintln!("Invalid port number: {v}");
                    return 1;
                }
            }
        }
        i += 1;
    }
    let backend = match file {
        None => Backend::Api(port),
        Some(path) => {
            if !path.exists() {
                eprintln!("[Error] File not found: {}", path.display());
                return 1;
            }
            let mut app = App::new(None);
            app.open_path(path.clone());
            if let Some((t, d)) = app.message.take() {
                eprintln!("[Error] {t} {d}");
                return 1;
            }
            Backend::File { app: Box::new(app), path, ms: EguiMeasure { ctx: ctx() } }
        }
    };
    eprintln!("UMLCraft MCP Server running on stdio");
    let stdin = std::io::stdin();
    match serve(backend, stdin.lock(), std::io::stdout()) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("[Error] {e}");
            1
        }
    }
}
