#![cfg(all(test, target_os = "linux"))]

use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt as XprotoConnectionExt, CreateWindowAux, EventMask, PropMode,
    WindowClass,
};

pub fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub struct X11TestEnv {
    child: Child,
    old_display: Option<String>,
    old_session_type: Option<String>,
}

impl X11TestEnv {
    pub fn new() -> Result<Option<Self>> {
        if Command::new("Xvfb")
            .arg("-help")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_err()
        {
            return Ok(None);
        }

        for display_num in 90..110 {
            let display = format!(":{display_num}");
            let lock_path = format!("/tmp/.X{display_num}-lock");
            let unix_socket = format!("/tmp/.X11-unix/X{display_num}");
            if Path::new(&lock_path).exists() || Path::new(&unix_socket).exists() {
                continue;
            }

            let child = Command::new("Xvfb")
                .arg(&display)
                .arg("-screen")
                .arg("0")
                .arg("1024x768x24")
                .arg("-nolisten")
                .arg("tcp")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .with_context(|| format!("Failed to launch Xvfb on {display}"))?;

            thread::sleep(Duration::from_millis(250));

            let old_display = std::env::var("DISPLAY").ok();
            let old_session_type = std::env::var("XDG_SESSION_TYPE").ok();
            std::env::set_var("DISPLAY", &display);
            std::env::set_var("XDG_SESSION_TYPE", "x11");

            return Ok(Some(Self {
                child,
                old_display,
                old_session_type,
            }));
        }

        anyhow::bail!("Failed to find a free Xvfb display")
    }

    pub fn create_window(&self, title: &str, app_class: &str) -> Result<()> {
        let (conn, screen_num) =
            x11rb::connect(None).context("Failed to connect to test Xvfb display")?;
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

        thread::sleep(Duration::from_millis(150));
        Ok(())
    }
}

impl Drop for X11TestEnv {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();

        match &self.old_display {
            Some(value) => std::env::set_var("DISPLAY", value),
            None => std::env::remove_var("DISPLAY"),
        }

        match &self.old_session_type {
            Some(value) => std::env::set_var("XDG_SESSION_TYPE", value),
            None => std::env::remove_var("XDG_SESSION_TYPE"),
        }
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
