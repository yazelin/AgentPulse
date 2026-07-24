use crate::hook_event::HookEvent;
use crate::mori_bridge::{manifest_json, sessions_json, sse_frame, MoriEvent};
use crate::session::AppState;
use log::{error, info};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, watch};

pub struct HookServer {
    port: u16,
}

impl HookServer {
    pub fn new() -> Self {
        Self { port: 0 }
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn start(
        &mut self,
        cue_tx: broadcast::Sender<MoriEvent>,
        snap_rx: watch::Receiver<AppState>,
    ) -> Result<mpsc::UnboundedReceiver<HookEvent>, ServerError> {
        if let Some(existing_port) = read_existing_port_file() {
            if is_port_listening(existing_port).await {
                return Err(ServerError::AnotherInstanceRunning(existing_port));
            }
        }

        let (tx, rx) = mpsc::unbounded_channel();

        for candidate_port in 19280..=19289u16 {
            match TcpListener::bind(format!("127.0.0.1:{candidate_port}")).await {
                Ok(listener) => {
                    self.port = candidate_port;
                    write_port_file(candidate_port);
                    info!("AgentPulse server listening on port {candidate_port}");

                    let tx = Arc::new(tx);
                    tokio::spawn(accept_loop(listener, tx, cue_tx.clone(), snap_rx.clone(), candidate_port));

                    return Ok(rx);
                }
                Err(_) => continue,
            }
        }

        Err(ServerError::NoAvailablePort)
    }
}

async fn accept_loop(
    listener: TcpListener,
    tx: Arc<mpsc::UnboundedSender<HookEvent>>,
    cue_tx: broadcast::Sender<MoriEvent>,
    snap_rx: watch::Receiver<AppState>,
    port: u16,
) {
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let tx = tx.clone();
                let cue_tx = cue_tx.clone();
                let snap_rx = snap_rx.clone();
                tokio::spawn(handle_client(stream, tx, cue_tx, snap_rx, port));
            }
            Err(e) => error!("Accept error: {e}"),
        }
    }
}

async fn handle_client(
    mut stream: tokio::net::TcpStream,
    tx: Arc<mpsc::UnboundedSender<HookEvent>>,
    cue_tx: broadcast::Sender<MoriEvent>,
    snap_rx: watch::Receiver<AppState>,
    port: u16,
) {
    let mut buf = vec![0u8; 65536];
    let n = match tokio::time::timeout(
        std::time::Duration::from_secs(2),
        stream.read(&mut buf),
    )
    .await
    {
        Ok(Ok(n)) if n > 0 => n,
        _ => return,
    };
    let data = &buf[..n];
    let (method, path) = parse_request_line(data);

    // ---- Mori body-interface read endpoints (GET) ----
    if method == "GET" {
        match path.as_str() {
            "/health" => {
                let _ = stream.write_all(http_ok("text/plain", "ok").as_bytes()).await;
            }
            "/manifest" => {
                let _ = stream
                    .write_all(http_ok("application/json", &manifest_json(port)).as_bytes())
                    .await;
            }
            "/sessions" => {
                let body = sessions_json(&snap_rx.borrow());
                let _ = stream
                    .write_all(http_ok("application/json", &body).as_bytes())
                    .await;
            }
            "/events" => {
                serve_sse(stream, cue_tx).await;
            }
            _ => {
                let _ = stream
                    .write_all(b"HTTP/1.1 404 Not Found\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                    .await;
            }
        }
        return;
    }

    // ---- existing hook entry (POST /hook/{provider}) — unchanged ----
    let provider = parse_provider(data);
    let response = if let Some(body_start) = find_body_start(data) {
        let body = &data[body_start..];
        match serde_json::from_slice::<crate::hook_event::RawHookEvent>(body) {
            Ok(raw) => {
                let mut event = raw.normalize(&provider);
                normalize_event_name(&mut event);
                if event.session_id.is_empty() {
                    event.session_id = format!("{}-default", event.provider);
                }
                let _ = tx.send(event);
                "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}"
            }
            Err(_) => "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        }
    } else {
        "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    };
    let _ = stream.write_all(response.as_bytes()).await;
}

/// "GET /sessions HTTP/1.1" → ("GET", "/sessions") (query stripped).
fn parse_request_line(data: &[u8]) -> (String, String) {
    let line = data
        .split(|&b| b == b'\r' || b == b'\n')
        .next()
        .map(|l| String::from_utf8_lossy(l).to_string())
        .unwrap_or_default();
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let raw_path = parts.next().unwrap_or("");
    let path = raw_path.split('?').next().unwrap_or("").to_string();
    (method, path)
}

fn http_ok(content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {ct}\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        ct = content_type,
        len = body.len(),
    )
}

/// SSE: keep-alive; write each broadcast MoriEvent as `data: {json}\n\n`.
async fn serve_sse(mut stream: tokio::net::TcpStream, cue_tx: broadcast::Sender<MoriEvent>) {
    let mut rx = cue_tx.subscribe();
    let head = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nAccess-Control-Allow-Origin: *\r\nConnection: keep-alive\r\n\r\n";
    if stream.write_all(head.as_bytes()).await.is_err() {
        return;
    }
    let _ = stream.write_all(b": connected\n\n").await;
    loop {
        match rx.recv().await {
            Ok(ev) => {
                let json = serde_json::to_string(&ev).unwrap_or_default();
                if stream.write_all(sse_frame(&json).as_bytes()).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(_)) => continue,
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
}

/// Parse provider from HTTP request line: "POST /hook/claude HTTP/1.1"
fn parse_provider(data: &[u8]) -> String {
    let request_line = data.split(|&b| b == b'\r' || b == b'\n')
        .next()
        .unwrap_or(b"");
    let line = String::from_utf8_lossy(request_line);

    // Extract path from "POST /hook/provider HTTP/1.1"
    if let Some(path_start) = line.find("/hook/") {
        let after = &line[path_start + 6..];
        if let Some(end) = after.find(|c: char| c == ' ' || c == '/' || c == '?') {
            return after[..end].to_string();
        }
        // No space found, take rest (shouldn't happen with valid HTTP)
        if let Some(end) = after.find(' ') {
            return after[..end].to_string();
        }
    }

    // Fallback: /hook without provider = claude (backward compat)
    "claude".to_string()
}

/// Normalize different CLI event names to a common set
fn normalize_event_name(event: &mut HookEvent) {
    let normalized = match event.hook_event_name.as_str() {
        // Antigravity CLI (agy). The sidecar injects the event name from the
        // hooks.json key (agy's stdin payload carries none). `PostToolUse` and
        // `Stop` are already standard PascalCase and pass through below.
        // `PreInvocation` fires before each model call → UserPromptSubmit,
        // which puts the session Working. (SessionStart would set Idle, so a
        // tool-less turn would never reach Working and its `Stop` wouldn't
        // emit Completed — no sound. UserPromptSubmit is robust to that.)
        // It is deliberately NOT mapped to Stop — only agy's real `Stop` ends
        // a turn.
        "PreInvocation" => "UserPromptSubmit",
        // GitHub Copilot CLI events (camelCase) → standard names
        "sessionStart" => "SessionStart",
        "sessionEnd" => "SessionEnd",
        "preToolUse" => "PreToolUse",
        "postToolUse" => "PostToolUse",
        "userPromptSubmitted" => "UserPromptSubmit",
        // agentStop = main agent finished (real completion).
        // subagentStop is intentionally NOT mapped to Stop: a single user
        // prompt can spawn a subagent, and subagentStop fires when the
        // child finishes while the parent is still working. Mapping it to
        // Stop would emit a mid-turn Completed and the parent's agentStop
        // would be swallowed (state already Idle). Fall through to no-op.
        "agentStop" => "Stop",
        "errorOccurred" => "Notification",
        // Already standard names (Claude + Codex use PascalCase)
        "SessionStart" | "SessionEnd" | "PreToolUse" | "PostToolUse" |
        "UserPromptSubmit" | "Stop" | "PermissionRequest" |
        "PostToolUseFailure" | "Notification" => event.hook_event_name.as_str(),
        other => other,
    };
    event.hook_event_name = normalized.to_string();
}

fn find_body_start(data: &[u8]) -> Option<usize> {
    let separator = b"\r\n\r\n";
    data.windows(4)
        .position(|w| w == separator)
        .map(|pos| pos + 4)
}

fn read_existing_port_file() -> Option<u16> {
    let home = dirs::home_dir()?;
    let path = home.join(".agentpulse").join("port");
    let content = std::fs::read_to_string(path).ok()?;
    content.trim().parse().ok()
}

async fn is_port_listening(port: u16) -> bool {
    match tokio::time::timeout(
        std::time::Duration::from_secs(1),
        tokio::net::TcpStream::connect(format!("127.0.0.1:{port}")),
    )
    .await
    {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

fn write_port_file(port: u16) {
    if let Some(home) = dirs::home_dir() {
        let dir = home.join(".agentpulse");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("port"), port.to_string());
    }
}

pub fn remove_port_file() {
    if let Some(home) = dirs::home_dir() {
        let _ = std::fs::remove_file(home.join(".agentpulse").join("port"));
    }
}

#[derive(Debug)]
pub enum ServerError {
    NoAvailablePort,
    AnotherInstanceRunning(u16),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAvailablePort => write!(f, "No available port in range 19280-19289"),
            Self::AnotherInstanceRunning(p) => {
                write!(f, "Another AgentPulse instance is running on port {p}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_request_line_get_strips_query() {
        let (m, p) = parse_request_line(b"GET /sessions?ts=123 HTTP/1.1\r\nHost: x\r\n\r\n");
        assert_eq!(m, "GET");
        assert_eq!(p, "/sessions");
    }

    #[test]
    fn parse_request_line_post_hook_path() {
        let (m, p) = parse_request_line(b"POST /hook/claude HTTP/1.1\r\n\r\n{}");
        assert_eq!(m, "POST");
        assert_eq!(p, "/hook/claude");
    }

    #[test]
    fn parse_request_line_empty_is_safe() {
        let (m, p) = parse_request_line(b"");
        assert_eq!(m, "");
        assert_eq!(p, "");
    }

    #[test]
    fn http_ok_content_length_is_byte_count_with_cors() {
        let body = "日本語"; // 3 chars / 9 bytes — Content-Length must be bytes.
        let resp = http_ok("application/json", body);
        assert!(resp.contains("Content-Length: 9"), "byte length, not char count");
        assert!(resp.contains("Access-Control-Allow-Origin: *"));
        assert!(resp.contains("Content-Type: application/json"));
        assert!(resp.ends_with(body));
    }
}
