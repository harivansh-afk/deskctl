use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use crate::cli::GlobalOpts;
use crate::core::protocol::{Request, Response};

fn socket_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("DESKTOP_CTL_SOCKET_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime).join("desktop-ctl");
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".desktop-ctl")
}

fn socket_path(opts: &GlobalOpts) -> PathBuf {
    if let Some(ref path) = opts.socket {
        return path.clone();
    }
    socket_dir().join(format!("{}.sock", opts.session))
}

fn pid_path(opts: &GlobalOpts) -> PathBuf {
    socket_dir().join(format!("{}.pid", opts.session))
}

fn try_connect(opts: &GlobalOpts) -> Option<UnixStream> {
    UnixStream::connect(socket_path(opts)).ok()
}

fn spawn_daemon(opts: &GlobalOpts) -> Result<()> {
    let exe = std::env::current_exe()
        .context("Failed to determine executable path")?;

    let sock_dir = socket_dir();
    std::fs::create_dir_all(&sock_dir)
        .context("Failed to create socket directory")?;

    let mut cmd = Command::new(exe);
    cmd.env("DESKTOP_CTL_DAEMON", "1")
        .env("DESKTOP_CTL_SESSION", &opts.session)
        .env("DESKTOP_CTL_SOCKET_PATH", socket_path(opts))
        .env("DESKTOP_CTL_PID_PATH", pid_path(opts))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    // Detach the daemon process on Unix
    unsafe {
        cmd.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    cmd.spawn().context("Failed to spawn daemon")?;
    Ok(())
}

fn ensure_daemon(opts: &GlobalOpts) -> Result<UnixStream> {
    // Try connecting first
    if let Some(stream) = try_connect(opts) {
        return Ok(stream);
    }

    // Spawn daemon
    spawn_daemon(opts)?;

    // Retry with backoff
    let max_retries = 20;
    let base_delay = Duration::from_millis(50);
    for i in 0..max_retries {
        thread::sleep(base_delay * (i + 1).min(4));
        if let Some(stream) = try_connect(opts) {
            return Ok(stream);
        }
    }

    bail!(
        "Failed to connect to daemon after {} retries.\n\
         Socket path: {}",
        max_retries,
        socket_path(opts).display()
    );
}

pub fn send_command(opts: &GlobalOpts, request: &Request) -> Result<Response> {
    let mut stream = ensure_daemon(opts)?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    // Send NDJSON request
    let json = serde_json::to_string(request)?;
    writeln!(stream, "{json}")?;
    stream.flush()?;

    // Read NDJSON response
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let response: Response = serde_json::from_str(line.trim())
        .context("Failed to parse daemon response")?;

    Ok(response)
}

pub fn start_daemon(opts: &GlobalOpts) -> Result<()> {
    if try_connect(opts).is_some() {
        println!("Daemon already running ({})", socket_path(opts).display());
        return Ok(());
    }
    spawn_daemon(opts)?;
    // Wait briefly and verify
    thread::sleep(Duration::from_millis(200));
    if try_connect(opts).is_some() {
        println!("Daemon started ({})", socket_path(opts).display());
    } else {
        bail!("Daemon failed to start");
    }
    Ok(())
}

pub fn stop_daemon(opts: &GlobalOpts) -> Result<()> {
    match try_connect(opts) {
        Some(mut stream) => {
            let req = Request::new("shutdown");
            let json = serde_json::to_string(&req)?;
            writeln!(stream, "{json}")?;
            stream.flush()?;
            println!("Daemon stopped");
        }
        None => {
            // Try to clean up stale socket
            let path = socket_path(opts);
            if path.exists() {
                std::fs::remove_file(&path)?;
                println!("Removed stale socket: {}", path.display());
            } else {
                println!("Daemon not running");
            }
        }
    }
    Ok(())
}

pub fn daemon_status(opts: &GlobalOpts) -> Result<()> {
    if try_connect(opts).is_some() {
        println!("Daemon running ({})", socket_path(opts).display());
    } else {
        println!("Daemon not running");
    }
    Ok(())
}
