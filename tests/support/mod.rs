#![cfg(target_os = "linux")]

use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use deskctl::cli::{connection, GlobalOpts};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt as XprotoConnectionExt, CreateWindowAux, EventMask, PropMode,
    WindowClass,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as X11WrapperConnectionExt;

pub fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub struct SessionEnvGuard {
    old_session_type: Option<String>,
}

impl SessionEnvGuard {
    pub fn prepare() -> Option<Self> {
        if std::env::var("DISPLAY")
            .ok()
            .filter(|value| !value.is_empty())
            .is_none()
        {
            return None;
        }

        let old_session_type = std::env::var("XDG_SESSION_TYPE").ok();
        std::env::set_var("XDG_SESSION_TYPE", "x11");
        Some(Self { old_session_type })
    }
}

impl Drop for SessionEnvGuard {
    fn drop(&mut self) {
        match &self.old_session_type {
            Some(value) => std::env::set_var("XDG_SESSION_TYPE", value),
            None => std::env::remove_var("XDG_SESSION_TYPE"),
        }
    }
}

pub struct FixtureWindow {
    conn: RustConnection,
    window: u32,
}

impl FixtureWindow {
    pub fn create(title: &str, app_class: &str) -> Result<Self> {
        let (conn, screen_num) =
            x11rb::connect(None).context("Failed to connect to the integration test display")?;
        let screen = &conn.setup().roots[screen_num];
        let window = conn.generate_id()?;

        conn.create_window(
            x11rb::COPY_DEPTH_FROM_PARENT,
            window,
            screen.root,
            10,
            10,
            320,
            180,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new()
                .background_pixel(screen.white_pixel)
                .event_mask(EventMask::EXPOSURE),
        )?;
        conn.change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            title.as_bytes(),
        )?;
        let class_bytes = format!("{app_class}\0{app_class}\0");
        conn.change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            class_bytes.as_bytes(),
        )?;
        conn.map_window(window)?;
        conn.flush()?;

        std::thread::sleep(std::time::Duration::from_millis(150));
        Ok(Self { conn, window })
    }
}

impl Drop for FixtureWindow {
    fn drop(&mut self) {
        let _ = self.conn.destroy_window(self.window);
        let _ = self.conn.flush();
    }
}

pub struct TestSession {
    pub opts: GlobalOpts,
    root: PathBuf,
}

impl TestSession {
    pub fn new(label: &str) -> Result<Self> {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("System clock is before the Unix epoch")?
            .as_nanos();
        let root = std::env::temp_dir().join(format!("deskctl-{label}-{suffix}"));
        std::fs::create_dir_all(&root)
            .with_context(|| format!("Failed to create {}", root.display()))?;

        Ok(Self {
            opts: GlobalOpts {
                socket: Some(root.join("deskctl.sock")),
                session: format!("{label}-{suffix}"),
                json: false,
            },
            root,
        })
    }

    pub fn socket_path(&self) -> &Path {
        self.opts
            .socket
            .as_deref()
            .expect("TestSession always has an explicit socket path")
    }

    pub fn create_stale_socket(&self) -> Result<()> {
        let listener = UnixListener::bind(self.socket_path())
            .with_context(|| format!("Failed to bind {}", self.socket_path().display()))?;
        drop(listener);
        Ok(())
    }

    pub fn start_daemon_cli(&self) -> Result<()> {
        let output = self.run_cli(["daemon", "start"])?;
        if output.status.success() {
            return Ok(());
        }

        bail!(
            "deskctl daemon start failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    pub fn run_cli<I, S>(&self, args: I) -> Result<Output>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let socket = self.socket_path();
        let mut command = Command::new(env!("CARGO_BIN_EXE_deskctl"));
        command
            .arg("--socket")
            .arg(socket)
            .arg("--session")
            .arg(&self.opts.session);

        for arg in args {
            command.arg(arg.as_ref());
        }

        command.output().with_context(|| {
            format!(
                "Failed to run {} against {}",
                env!("CARGO_BIN_EXE_deskctl"),
                socket.display()
            )
        })
    }
}

impl Drop for TestSession {
    fn drop(&mut self) {
        let _ = connection::stop_daemon(&self.opts);
        if self.socket_path().exists() {
            let _ = std::fs::remove_file(self.socket_path());
        }
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub fn deskctl_tmp_screenshot_count() -> usize {
    std::fs::read_dir("/tmp")
        .ok()
        .into_iter()
        .flat_map(|iter| iter.filter_map(Result::ok))
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .map(|name| name.starts_with("deskctl-") && name.ends_with(".png"))
                .unwrap_or(false)
        })
        .count()
}

pub fn successful_json_response(output: Output) -> Result<serde_json::Value> {
    if !output.status.success() {
        return Err(anyhow!(
            "deskctl command failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    serde_json::from_slice(&output.stdout).context("Failed to parse JSON output from deskctl")
}
