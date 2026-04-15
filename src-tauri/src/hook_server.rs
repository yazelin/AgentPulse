use crate::hook_event::HookEvent;
use log::{error, info};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

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

    /// Start the server, trying ports 19280-19289. Returns the event receiver.
    pub async fn start(
        &mut self,
    ) -> Result<mpsc::UnboundedReceiver<HookEvent>, ServerError> {
        // Check if another instance is running
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
                    info!("ClaudePulse server listening on port {candidate_port}");

                    let tx = Arc::new(tx);
                    tokio::spawn(accept_loop(listener, tx));

                    return Ok(rx);
                }
                Err(_) => continue,
            }
        }

        Err(ServerError::NoAvailablePort)
    }
}

async fn accept_loop(listener: TcpListener, tx: Arc<mpsc::UnboundedSender<HookEvent>>) {
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let tx = tx.clone();
                tokio::spawn(handle_client(stream, tx));
            }
            Err(e) => {
                error!("Accept error: {e}");
            }
        }
    }
}

async fn handle_client(
    mut stream: tokio::net::TcpStream,
    tx: Arc<mpsc::UnboundedSender<HookEvent>>,
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
    let response = if let Some(body_start) = find_body_start(data) {
        let body = &data[body_start..];
        match serde_json::from_slice::<HookEvent>(body) {
            Ok(event) => {
                let _ = tx.send(event);
                "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}"
            }
            Err(_) => {
                "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            }
        }
    } else {
        "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    };

    let _ = stream.write_all(response.as_bytes()).await;
}

fn find_body_start(data: &[u8]) -> Option<usize> {
    let separator = b"\r\n\r\n";
    data.windows(4)
        .position(|w| w == separator)
        .map(|pos| pos + 4)
}

fn read_existing_port_file() -> Option<u16> {
    let home = dirs::home_dir()?;
    let path = home.join(".ccani").join("port");
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
        let dir = home.join(".ccani");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("port"), port.to_string());
    }
}

pub fn remove_port_file() {
    if let Some(home) = dirs::home_dir() {
        let _ = std::fs::remove_file(home.join(".ccani").join("port"));
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
                write!(f, "Another ClaudePulse instance is running on port {p}")
            }
        }
    }
}
