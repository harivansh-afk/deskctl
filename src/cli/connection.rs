use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use anyhow::{bail, Context, Result};

use crate::cli::GlobalOpts;
use crate::core::doctor::{run as run_doctor_report, DoctorReport};
use crate::core::paths::{pid_path_for_session, socket_dir, socket_path_for_session};
use crate::core::protocol::{Request, Response};

fn socket_path(opts: &GlobalOpts) -> PathBuf {
    if let Some(ref path) = opts.socket {
        return path.clone();
    }
    socket_path_for_session(&opts.session)
}

fn pid_path(opts: &GlobalOpts) -> PathBuf {
    pid_path_for_session(&opts.session)
}

fn connect_socket(path: &Path) -> Result<UnixStream> {
    UnixStream::connect(path).with_context(|| format!("Failed to connect to {}", path.display()))
}

fn is_stale_socket_error(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound
    )
}

fn cleanup_stale_socket(opts: &GlobalOpts) -> Result<bool> {
    let path = socket_path(opts);
    if !path.exists() {
        return Ok(false);
    }

    match UnixStream::connect(&path) {
        Ok(_) => Ok(false),
        Err(error) if is_stale_socket_error(&error) => {
            std::fs::remove_file(&path)
                .with_context(|| format!("Failed to remove stale socket {}", path.display()))?;
            Ok(true)
        }
        Err(error) => Err(error)
            .with_context(|| format!("Failed to inspect daemon socket {}", path.display())),
    }
}

fn spawn_daemon(opts: &GlobalOpts) -> Result<()> {
    let exe = std::env::current_exe().context("Failed to determine executable path")?;

    let sock_dir = socket_dir();
    std::fs::create_dir_all(&sock_dir).context("Failed to create socket directory")?;

    let mut cmd = Command::new(exe);
    cmd.env("DESKCTL_DAEMON", "1")
        .env("DESKCTL_SESSION", &opts.session)
        .env("DESKCTL_SOCKET_PATH", socket_path(opts))
        .env("DESKCTL_PID_PATH", pid_path(opts))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    unsafe {
        cmd.pre_exec(|| {
            libc::setsid();
            Ok(())
        });
    }

    cmd.spawn().context("Failed to spawn daemon")?;
    Ok(())
}

fn request_read_timeout(request: &Request) -> Duration {
    let default_timeout = Duration::from_secs(30);
    match request.action.as_str() {
        "wait-window" | "wait-focus" => {
            let wait_timeout = request
                .extra
                .get("timeout_ms")
                .and_then(|value| value.as_u64())
                .unwrap_or(10_000);
            Duration::from_millis(wait_timeout.saturating_add(5_000))
        }
        _ => default_timeout,
    }
}

fn send_request_over_stream(mut stream: UnixStream, request: &Request) -> Result<Response> {
    stream.set_read_timeout(Some(request_read_timeout(request)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let json = serde_json::to_string(request)?;
    writeln!(stream, "{json}")?;
    stream.flush()?;

    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;

    serde_json::from_str(line.trim()).context("Failed to parse daemon response")
}

fn ping_daemon(opts: &GlobalOpts) -> Result<()> {
    let response =
        send_request_over_stream(connect_socket(&socket_path(opts))?, &Request::new("ping"))?;
    if response.success {
        Ok(())
    } else {
        bail!(
            "{}",
            response
                .error
                .unwrap_or_else(|| "Daemon health probe failed".to_string())
        );
    }
}

fn ensure_daemon(opts: &GlobalOpts) -> Result<UnixStream> {
    if ping_daemon(opts).is_ok() {
        return connect_socket(&socket_path(opts));
    }

    let removed_stale_socket = cleanup_stale_socket(opts)?;
    if removed_stale_socket && ping_daemon(opts).is_ok() {
        return connect_socket(&socket_path(opts));
    }

    spawn_daemon(opts)?;

    let max_retries = 20;
    let base_delay = Duration::from_millis(50);
    for attempt in 0..max_retries {
        thread::sleep(base_delay * (attempt + 1).min(4));
        if ping_daemon(opts).is_ok() {
            return connect_socket(&socket_path(opts));
        }
    }

    bail!(
        "Failed to start a healthy daemon after {} retries.\nSocket path: {}",
        max_retries,
        socket_path(opts).display()
    );
}

pub fn send_command(opts: &GlobalOpts, request: &Request) -> Result<Response> {
    send_request_over_stream(ensure_daemon(opts)?, request)
}

pub fn run_doctor(opts: &GlobalOpts) -> Result<()> {
    let report = run_doctor_report(&socket_path(opts));
    print_doctor_report(&report, opts.json)?;
    if !report.healthy {
        std::process::exit(1);
    }
    Ok(())
}

pub fn start_daemon(opts: &GlobalOpts) -> Result<()> {
    if ping_daemon(opts).is_ok() {
        println!("Daemon already running ({})", socket_path(opts).display());
        return Ok(());
    }

    if cleanup_stale_socket(opts)? {
        println!("Removed stale socket: {}", socket_path(opts).display());
    }

    spawn_daemon(opts)?;

    let max_retries = 20;
    for attempt in 0..max_retries {
        thread::sleep(Duration::from_millis(50 * (attempt + 1).min(4) as u64));
        if ping_daemon(opts).is_ok() {
            println!("Daemon started ({})", socket_path(opts).display());
            return Ok(());
        }
    }

    bail!(
        "Daemon failed to become healthy.\nSocket path: {}",
        socket_path(opts).display()
    );
}

pub fn stop_daemon(opts: &GlobalOpts) -> Result<()> {
    let path = socket_path(opts);
    match UnixStream::connect(&path) {
        Ok(stream) => {
            let response = send_request_over_stream(stream, &Request::new("shutdown"))?;
            if response.success {
                println!("Daemon stopped");
            } else {
                bail!(
                    "{}",
                    response
                        .error
                        .unwrap_or_else(|| "Failed to stop daemon".to_string())
                );
            }
        }
        Err(error) if is_stale_socket_error(&error) => {
            if path.exists() {
                std::fs::remove_file(&path)?;
                println!("Removed stale socket: {}", path.display());
            } else {
                println!("Daemon not running");
            }
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Failed to inspect daemon socket {}", path.display()));
        }
    }
    Ok(())
}

pub fn daemon_status(opts: &GlobalOpts) -> Result<()> {
    let path = socket_path(opts);
    match ping_daemon(opts) {
        Ok(()) => println!("Daemon running ({})", path.display()),
        Err(_) if path.exists() => {
            println!("Daemon socket exists but is unhealthy ({})", path.display())
        }
        Err(_) => println!("Daemon not running"),
    }
    Ok(())
}

fn print_doctor_report(report: &DoctorReport, json_output: bool) -> Result<()> {
    if json_output {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }

    println!(
        "deskctl doctor: {}",
        if report.healthy {
            "healthy"
        } else {
            "issues found"
        }
    );
    for check in &report.checks {
        let status = if check.ok { "OK" } else { "FAIL" };
        println!("[{status}] {}: {}", check.name, check.details);
        if let Some(fix) = &check.fix {
            println!("       fix: {fix}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{cleanup_stale_socket, socket_path};
    use crate::cli::GlobalOpts;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn cleanup_stale_socket_removes_refused_socket_path() {
        let temp = std::env::temp_dir().join(format!(
            "deskctl-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp).unwrap();
        let opts = GlobalOpts {
            socket: Some(temp.join("stale.sock")),
            session: "test".to_string(),
            json: false,
        };

        let listener = std::os::unix::net::UnixListener::bind(socket_path(&opts)).unwrap();
        drop(listener);

        assert!(cleanup_stale_socket(&opts).unwrap());
        assert!(!socket_path(&opts).exists());

        let _ = std::fs::remove_dir_all(&temp);
    }
}
