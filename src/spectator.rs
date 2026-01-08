#[cfg(feature = "spectator")]
use serde::Serialize;
#[cfg(feature = "spectator")]
use std::io::{Read, Write};
#[cfg(feature = "spectator")]
use std::net::TcpListener;
#[cfg(feature = "spectator")]
use std::sync::mpsc;
#[cfg(feature = "spectator")]
use std::thread;
#[cfg(feature = "spectator")]
use std::time::Duration;
#[cfg(feature = "spectator")]
use tungstenite::{accept, Message, WebSocket};

#[cfg(feature = "spectator")]
#[derive(Serialize)]
pub struct GameSnapshot {
    pub width: i32,
    pub height: i32,
    pub snake: Vec<(i32, i32)>,
    pub food: Option<(i32, i32)>,
    pub score: usize,
    pub state: String,
    pub tick: u64,
}

#[cfg(feature = "spectator")]
pub struct SpectatorHandle {
    tx: mpsc::Sender<GameSnapshot>,
}

#[cfg(feature = "spectator")]
impl SpectatorHandle {
    pub fn send(&self, snapshot: GameSnapshot) {
        let _ = self.tx.send(snapshot);
    }
}

#[cfg(feature = "spectator")]
pub fn start(addr: &str) -> SpectatorHandle {
    let (tx, rx) = mpsc::channel::<GameSnapshot>();
    let addr = addr.to_string();

    thread::spawn(move || {
        let listener = TcpListener::bind(&addr).expect("Failed to bind spectator socket");
        listener.set_nonblocking(true).expect("Failed to set nonblocking spectator socket");

        let mut clients: Vec<WebSocket<std::net::TcpStream>> = Vec::new();

        loop {
            while let Ok((stream, _)) = listener.accept() {
                if let Ok(mut ws) = accept(stream) {
                    let _ = ws.get_mut().set_nodelay(true);
                    clients.push(ws);
                }
            }

            match rx.recv_timeout(Duration::from_millis(16)) {
                Ok(snapshot) => {
                    let payload = serde_json::to_string(&snapshot).unwrap_or_default();
                    clients.retain_mut(|client| client
                        .send(Message::Text(payload.clone().into()))
                        .is_ok()
                    );
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    SpectatorHandle { tx }
}

#[cfg(feature = "spectator")]
pub fn start_http(addr: &str) {
    let addr = addr.to_string();
    thread::spawn(move || {
        let listener = TcpListener::bind(&addr).expect("Failed to bind spectator HTTP socket");
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };

            let mut buf = [0u8; 1024];
            let bytes = stream.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..bytes]);
            let path = req
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap_or("/");

            let body = match path {
                "/" | "/spectator.html" => include_str!("../spectator.html"),
                "/health" => "ok",
                _ => "not found",
            };

            let status = if path == "/" || path == "/spectator.html" || path == "/health" {
                "200 OK"
            } else {
                "404 Not Found"
            };

            let content_type = if path == "/health" {
                "text/plain; charset=utf-8"
            } else {
                "text/html; charset=utf-8"
            };

            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.as_bytes().len()
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });
}
