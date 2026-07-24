// Sidecar binary invoked by CLI hook configs. Reads the event JSON from
// stdin and forwards it as an HTTP POST to the running AgentPulse server.
//
// Usage: agent-pulse-hook <provider_id>
//
// Shell-agnostic by design: no bash, no PowerShell, no cmd syntax. Any
// host CLI that can spawn a process (on any OS) can invoke this. Errors
// are swallowed so a hook misfire never breaks the parent CLI.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

const DEFAULT_PORT: u16 = 19280;
const TIMEOUT: Duration = Duration::from_secs(2);

fn main() {
    let mut args = std::env::args().skip(1);
    let provider = args.next().unwrap_or_else(|| "unknown".to_string());
    // Optional 2nd arg: event name. Antigravity's (agy) stdin payload has no
    // event-name field — the event is implied by the hooks.json key — so it is
    // passed here and injected into the body. Its presence also signals a
    // synchronous-hook host that parses stdout as the hook result.
    let event = args.next();

    let mut body = String::new();
    let _ = std::io::stdin().read_to_string(&mut body);

    if let Some(ref ev) = event {
        body = inject_event(&body, ev);
    }

    let port = read_port().unwrap_or(DEFAULT_PORT);
    let _ = post(port, &provider, &body);

    // Synchronous-hook hosts (agy) parse stdout as the hook result. The events
    // we register there (PreInvocation / PostToolUse / Stop) all accept an
    // empty object: no injected steps, no decision → default behavior. Only
    // emit when an event arg was passed, so fire-and-forget providers (Claude
    // et al., which may interpret hook stdout) stay silent.
    if event.is_some() {
        println!("{{}}");
    }
}

/// Insert `"hook_event_name": <event>` into the JSON body. Falls back to the
/// original body verbatim if it isn't a JSON object.
fn inject_event(body: &str, event: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(serde_json::Value::Object(mut map)) => {
            map.insert(
                "hook_event_name".into(),
                serde_json::Value::String(event.to_string()),
            );
            serde_json::Value::Object(map).to_string()
        }
        _ => body.to_string(),
    }
}

fn read_port() -> Option<u16> {
    let home = dirs::home_dir()?;
    let content = std::fs::read_to_string(home.join(".agentpulse").join("port")).ok()?;
    content.trim().parse().ok()
}

fn post(port: u16, provider: &str, body: &str) -> std::io::Result<()> {
    let addr: SocketAddr = format!("127.0.0.1:{port}")
        .parse()
        .expect("valid socket addr");

    let mut stream = TcpStream::connect_timeout(&addr, TIMEOUT)?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    stream.set_read_timeout(Some(TIMEOUT))?;

    let request = format!(
        "POST /hook/{provider} HTTP/1.0\r\n\
         Host: 127.0.0.1\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        body.len()
    );

    stream.write_all(request.as_bytes())?;
    stream.flush()?;

    let mut discard = [0u8; 64];
    let _ = stream.read(&mut discard);
    Ok(())
}
