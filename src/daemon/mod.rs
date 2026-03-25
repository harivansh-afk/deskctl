mod handler;
mod state;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::Mutex;

use crate::core::session;
use state::DaemonState;

pub fn run() -> Result<()> {
    // Validate session before starting
    session::detect_session()?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async_run())
}

async fn async_run() -> Result<()> {
    let socket_path = std::env::var("DESKTOP_CTL_SOCKET_PATH")
        .map(PathBuf::from)
        .context("DESKTOP_CTL_SOCKET_PATH not set")?;

    let pid_path = std::env::var("DESKTOP_CTL_PID_PATH")
        .map(PathBuf::from)
        .ok();

    // Clean up stale socket
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    // Write PID file
    if let Some(ref pid_path) = pid_path {
        std::fs::write(pid_path, std::process::id().to_string())?;
    }

    let listener = UnixListener::bind(&socket_path)
        .context(format!("Failed to bind socket: {}", socket_path.display()))?;

    let session = std::env::var("DESKTOP_CTL_SESSION").unwrap_or_else(|_| "default".to_string());
    let state = Arc::new(Mutex::new(
        DaemonState::new(session, socket_path.clone())
            .context("Failed to initialize daemon state")?
    ));

    let shutdown = Arc::new(tokio::sync::Notify::new());
    let shutdown_clone = shutdown.clone();

    // Accept loop
    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, _addr)) => {
                        let state = state.clone();
                        let shutdown = shutdown.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, state, shutdown).await {
                                eprintln!("Connection error: {e}");
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Accept error: {e}");
                    }
                }
            }
            _ = shutdown_clone.notified() => {
                break;
            }
        }
    }

    // Cleanup
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }
    if let Some(ref pid_path) = pid_path {
        let _ = std::fs::remove_file(pid_path);
    }

    Ok(())
}

async fn handle_connection(
    stream: tokio::net::UnixStream,
    state: Arc<Mutex<DaemonState>>,
    shutdown: Arc<tokio::sync::Notify>,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    reader.read_line(&mut line).await?;
    let line = line.trim();
    if line.is_empty() {
        return Ok(());
    }

    let request: crate::core::protocol::Request = serde_json::from_str(line)?;

    // Handle shutdown specially - notify before writing so the accept loop
    // exits even if the client has already closed the connection.
    if request.action == "shutdown" {
        shutdown.notify_one();
        let response = crate::core::protocol::Response::ok(
            serde_json::json!({"message": "Shutting down"})
        );
        let json = serde_json::to_string(&response)?;
        // Ignore write errors - client may have already closed the connection.
        let _ = writer.write_all(format!("{json}\n").as_bytes()).await;
        let _ = writer.flush().await;
        return Ok(());
    }

    let response = handler::handle_request(&request, &state).await;
    let json = serde_json::to_string(&response)?;
    writer.write_all(format!("{json}\n").as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}
