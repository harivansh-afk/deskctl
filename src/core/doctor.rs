use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use serde::Serialize;

use crate::backend::{x11::X11Backend, DesktopBackend};
use crate::core::protocol::{Request, Response};
use crate::core::session::detect_session;

#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub healthy: bool,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Serialize)]
pub struct DoctorCheck {
    pub name: String,
    pub ok: bool,
    pub details: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
}

pub fn run(socket_path: &Path) -> DoctorReport {
    let mut checks = Vec::new();

    let display = std::env::var("DISPLAY").ok();
    checks.push(match display {
        Some(ref value) if !value.is_empty() => check_ok("display", format!("DISPLAY={value}")),
        _ => check_fail(
            "display",
            "DISPLAY is not set".to_string(),
            "Export DISPLAY to point at the active X11 server.".to_string(),
        ),
    });

    checks.push(match detect_session() {
        Ok(_) => check_ok("session", "X11 session detected".to_string()),
        Err(error) => check_fail(
            "session",
            error.to_string(),
            "Run deskctl inside an X11 session. Wayland is not supported in this phase."
                .to_string(),
        ),
    });

    let mut backend = match X11Backend::new() {
        Ok(backend) => {
            checks.push(check_ok(
                "backend",
                "Connected to the X11 backend successfully".to_string(),
            ));
            Some(backend)
        }
        Err(error) => {
            checks.push(check_fail(
                "backend",
                error.to_string(),
                "Ensure the X server is reachable and the current session can access it."
                    .to_string(),
            ));
            None
        }
    };

    if let Some(backend) = backend.as_mut() {
        checks.push(match backend.list_windows() {
            Ok(windows) => check_ok(
                "window-enumeration",
                format!("Enumerated {} visible windows", windows.len()),
            ),
            Err(error) => check_fail(
                "window-enumeration",
                error.to_string(),
                "Verify the desktop session exposes EWMH window metadata and the X11 connection is healthy."
                    .to_string(),
            ),
        });

        checks.push(match backend.capture_screenshot() {
            Ok(image) => check_ok(
                "screenshot",
                format!(
                    "Captured {}x{} desktop image",
                    image.width(),
                    image.height()
                ),
            ),
            Err(error) => check_fail(
                "screenshot",
                error.to_string(),
                "Verify the X11 session permits desktop capture on the active display.".to_string(),
            ),
        });
    } else {
        checks.push(check_fail(
            "window-enumeration",
            "Skipped because backend initialization failed".to_string(),
            "Fix the X11 backend error before retrying.".to_string(),
        ));
        checks.push(check_fail(
            "screenshot",
            "Skipped because backend initialization failed".to_string(),
            "Fix the X11 backend error before retrying.".to_string(),
        ));
    }

    checks.push(check_socket_dir(socket_path));
    checks.push(check_daemon_socket(socket_path));

    let healthy = checks.iter().all(|check| check.ok);
    DoctorReport { healthy, checks }
}

fn check_socket_dir(socket_path: &Path) -> DoctorCheck {
    let Some(socket_dir) = socket_path.parent() else {
        return check_fail(
            "socket-dir",
            format!(
                "Socket path {} has no parent directory",
                socket_path.display()
            ),
            "Use a socket path inside a writable directory.".to_string(),
        );
    };

    match std::fs::create_dir_all(socket_dir) {
        Ok(()) => check_ok(
            "socket-dir",
            format!("Socket directory is ready at {}", socket_dir.display()),
        ),
        Err(error) => check_fail(
            "socket-dir",
            error.to_string(),
            format!("Ensure {} exists and is writable.", socket_dir.display()),
        ),
    }
}

fn check_daemon_socket(socket_path: &Path) -> DoctorCheck {
    if !socket_path.exists() {
        return check_ok(
            "daemon-socket",
            format!("No stale socket found at {}", socket_path.display()),
        );
    }

    match ping_socket(socket_path) {
        Ok(()) => check_ok(
            "daemon-socket",
            format!("Daemon is healthy at {}", socket_path.display()),
        ),
        Err(error) => check_fail(
            "daemon-socket",
            error.to_string(),
            format!(
                "Remove the stale socket at {} or run `deskctl daemon stop`.",
                socket_path.display()
            ),
        ),
    }
}

fn ping_socket(socket_path: &Path) -> Result<()> {
    let mut stream = UnixStream::connect(socket_path)?;
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;
    stream.set_write_timeout(Some(Duration::from_secs(1)))?;

    let request = Request::new("ping");
    let json = serde_json::to_string(&request)?;
    writeln!(stream, "{json}")?;
    stream.flush()?;

    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let response: Response = serde_json::from_str(line.trim())?;

    if response.success {
        Ok(())
    } else {
        anyhow::bail!(
            "{}",
            response
                .error
                .unwrap_or_else(|| "Daemon health probe failed".to_string())
        )
    }
}

fn check_ok(name: &str, details: String) -> DoctorCheck {
    DoctorCheck {
        name: name.to_string(),
        ok: true,
        details,
        fix: None,
    }
}

fn check_fail(name: &str, details: String, fix: String) -> DoctorCheck {
    DoctorCheck {
        name: name.to_string(),
        ok: false,
        details,
        fix: Some(fix),
    }
}
