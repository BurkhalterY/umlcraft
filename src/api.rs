//! The local HTTP API (main-process/api-server.js): a small server on 127.0.0.1, port 58321 by
//! default, that tools such as an MCP server use to read diagrams and to generate one from Mermaid.
//!
//! `settings.json` in UMLCraft's settings folder can turn it off (`"apiServer": false`) or move it
//! (`"apiServerPort": 58322`). It starts with the window, never for command-line commands. Requests
//! are answered by the window, on its own thread, through a channel.

use crate::app::App;
use crate::views::Measure;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

pub const DEFAULT_PORT: u16 = 58321;

/// A request for the window: the path, the JSON body, and where to send (status, JSON text).
pub struct ApiRequest {
    pub path: String,
    pub body: Value,
    pub reply: Sender<(u16, String)>,
}

/// Whether to start, and on which port, from `settings.json` (api-server.js:12-19).
pub fn settings(dir: &Path) -> (bool, u16) {
    let v: Value = std::fs::read_to_string(dir.join("settings.json")).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or(Value::Null);
    let on = v.get("apiServer").and_then(|x| x.as_bool()).unwrap_or(true);
    let port = v.get("apiServerPort").and_then(|x| x.as_u64()).filter(|p| *p > 0 && *p < 65536).map(|p| p as u16).unwrap_or(DEFAULT_PORT);
    (on, port)
}

/// Start the server; requests that need the model come out of the returned channel.
pub fn start(port: u16, ctx: egui::Context) -> Option<Receiver<ApiRequest>> {
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| eprintln!("[UMLCraft] API server not started on port {port}: {e}")).ok()?;
    let (tx, rx) = channel::<ApiRequest>();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let _ = serve(stream, &tx, &ctx);
        }
    });
    Some(rx)
}

fn respond(mut s: &TcpStream, status: u16, ctype: &str, body: &str) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Internal Server Error",
    };
    write!(s, "HTTP/1.1 {status} {reason}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())?;
    s.flush()
}

fn serve(stream: TcpStream, tx: &Sender<ApiRequest>, ctx: &egui::Context) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    let mut r = BufReader::new(&stream);
    let mut line = String::new();
    r.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let (method, path) = (parts.next().unwrap_or("").to_string(), parts.next().unwrap_or("/").to_string());
    let mut len = 0usize;
    let mut json_body = false;
    let mut local_host = false;
    loop {
        let mut h = String::new();
        if r.read_line(&mut h)? == 0 || h.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            if k.trim().eq_ignore_ascii_case("content-length") {
                len = v.trim().parse().unwrap_or(0);
            }
            if k.trim().eq_ignore_ascii_case("host") {
                let host = v.trim().rsplit_once(':').map(|x| x.0).unwrap_or(v.trim()).to_ascii_lowercase();
                local_host = matches!(host.as_str(), "127.0.0.1" | "localhost" | "[::1]");
            }
            if k.trim().eq_ignore_ascii_case("content-type") {
                json_body = v.trim().to_ascii_lowercase().starts_with("application/json");
            }
        }
    }
    let mut body = vec![0u8; len.min(16 * 1024 * 1024)];
    r.read_exact(&mut body)?;
    // a name other than localhost would be a page reaching it through DNS rebinding
    if !local_host {
        return respond(&stream, 400, "application/json", &json!({"error": "Host must be localhost"}).to_string());
    }
    if method == "GET" && path == "/" {
        return respond(&stream, 200, "text/plain", "Hello from UMLCraft API Server!");
    }
    if method != "POST" {
        return respond(&stream, 404, "application/json", &json!({"error": "Not found"}).to_string());
    }
    // only JSON requests: a web page cannot send those to this server without a CORS preflight,
    // which it never grants
    if !json_body {
        return respond(&stream, 400, "application/json", &json!({"error": "Content-Type must be application/json"}).to_string());
    }
    let body: Value = if body.is_empty() { json!({}) } else { serde_json::from_slice(&body).unwrap_or(Value::Null) };
    let (rtx, rrx) = channel();
    if tx.send(ApiRequest { path, body, reply: rtx }).is_err() {
        return respond(&stream, 500, "application/json", &json!({"error": "The window is closed"}).to_string());
    }
    ctx.request_repaint();
    match rrx.recv_timeout(Duration::from_secs(60)) {
        Ok((status, text)) => respond(&stream, status, "application/json", &text),
        Err(_) => respond(&stream, 500, "application/json", &json!({"error": "No answer from the window"}).to_string()),
    }
}

impl App {
    /// A diagram's record: id, type (class name), name and description (default-commands.js:1389-1418).
    fn diagram_info(&self, d: crate::model::Eid) -> Value {
        json!({"id": self.repo.id(d), "type": self.repo.ty(d), "name": self.repo.name(d), "description": self.repo.str(d, "documentation")})
    }

    /// Answer one API request: (status, JSON text) (api-server.js:34-146).
    pub fn api_answer(&mut self, path: &str, body: &Value, ms: &dyn Measure) -> (u16, String) {
        match path {
            "/generate_diagram" => {
                let Some(code) = body.get("code").and_then(|c| c.as_str()) else {
                    return (400, json!({"error": "Invalid request", "details": [{"path": ["code"], "message": "Required: a string"}]}).to_string());
                };
                match self.generate_mermaid(code, ms) {
                    Ok(_) => (200, json!({"success": true}).to_string()),
                    Err(e) => (500, json!({"error": e}).to_string()),
                }
            }
            "/get_all_diagrams_info" => {
                let all: Vec<Value> = self.print_diagrams(true).into_iter().map(|d| self.diagram_info(d)).collect();
                (200, Value::Array(all).to_string())
            }
            "/get_current_diagram_info" => {
                let v = self.diagram.filter(|d| self.repo.alive(*d)).map(|d| self.diagram_info(d)).unwrap_or(Value::Null);
                (200, v.to_string())
            }
            "/get_diagram_image_by_id" => {
                let id = body.get("diagramId").and_then(|x| x.as_str()).unwrap_or("");
                let d = self.repo.by_id.get(id).copied().filter(|d| self.repo.alive(*d) && self.is_diagram(*d));
                let Some(d) = d else {
                    return (500, json!({"error": "Diagram not found or invalid ID"}).to_string());
                };
                match crate::export::render(self, d, ms, crate::export::Format::Png, 1.0) {
                    Ok(png) => (200, json!({"success": true, "data": crate::model::base64(&png), "mimeType": "image/png"}).to_string()),
                    Err(e) => (500, json!({"error": e}).to_string()),
                }
            }
            _ => (404, json!({"error": "Not found"}).to_string()),
        }
    }

    /// Answer the requests waiting for the window.
    pub fn api_poll(&mut self, ctx: &egui::Context) {
        let Some(rx) = self.api_rx.take() else { return };
        let ms = crate::canvas::EguiMeasure { ctx: ctx.clone() };
        while let Ok(req) = rx.try_recv() {
            let answer = self.api_answer(&req.path, &req.body, &ms);
            let _ = req.reply.send(answer);
        }
        self.api_rx = Some(rx);
    }
}
