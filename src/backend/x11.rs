use anyhow::{Context, Result};
use enigo::{
    Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings,
};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    ClientMessageEvent, ConfigureWindowAux, ConnectionExt as XprotoConnectionExt,
    EventMask,
};
use x11rb::rust_connection::RustConnection;

use super::annotate::annotate_screenshot;
use crate::core::types::{Snapshot, WindowInfo};

pub struct X11Backend {
    enigo: Enigo,
    conn: RustConnection,
    root: u32,
}

impl X11Backend {
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| anyhow::anyhow!("Failed to initialize enigo: {e}"))?;
        let (conn, screen_num) = x11rb::connect(None)
            .context("Failed to connect to X11 server")?;
        let root = conn.setup().roots[screen_num].root;
        Ok(Self { enigo, conn, root })
    }
}

impl super::DesktopBackend for X11Backend {
    fn snapshot(&mut self, annotate: bool) -> Result<Snapshot> {
        // Get z-ordered window list via xcap (topmost first internally)
        let windows = xcap::Window::all().context("Failed to enumerate windows")?;

        // Get primary monitor for screenshot
        let monitors = xcap::Monitor::all().context("Failed to enumerate monitors")?;
        let monitor = monitors.into_iter().next().context("No monitor found")?;

        let mut image = monitor
            .capture_image()
            .context("Failed to capture screenshot")?;

        // Build window info list
        let mut window_infos = Vec::new();
        let mut ref_counter = 1usize;

        for win in &windows {
            // Each xcap method returns XCapResult<T> - skip windows where metadata fails
            let title = win.title().unwrap_or_default();
            let app_name = win.app_name().unwrap_or_default();

            // Skip windows with empty titles and app names (desktop, panels, etc.)
            if title.is_empty() && app_name.is_empty() {
                continue;
            }

            let xcb_id = win.id().unwrap_or(0);
            let x = win.x().unwrap_or(0);
            let y = win.y().unwrap_or(0);
            let width = win.width().unwrap_or(0);
            let height = win.height().unwrap_or(0);
            let focused = win.is_focused().unwrap_or(false);
            let minimized = win.is_minimized().unwrap_or(false);

            let ref_id = format!("w{ref_counter}");
            ref_counter += 1;

            window_infos.push(WindowInfo {
                ref_id,
                xcb_id,
                title,
                app_name,
                x,
                y,
                width,
                height,
                focused,
                minimized,
            });
        }

        // Annotate if requested - draw bounding boxes and @wN labels
        if annotate {
            annotate_screenshot(&mut image, &window_infos);
        }

        // Save screenshot
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let screenshot_path = format!("/tmp/desktop-ctl-{timestamp}.png");
        image
            .save(&screenshot_path)
            .context("Failed to save screenshot")?;

        Ok(Snapshot {
            screenshot: screenshot_path,
            windows: window_infos,
        })
    }

    fn focus_window(&mut self, xcb_id: u32) -> Result<()> {
        // Use _NET_ACTIVE_WINDOW client message (avoids focus-stealing prevention)
        let net_active = self
            .conn
            .intern_atom(false, b"_NET_ACTIVE_WINDOW")?
            .reply()
            .context("Failed to intern _NET_ACTIVE_WINDOW atom")?
            .atom;

        let event = ClientMessageEvent {
            response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
            format: 32,
            sequence: 0,
            window: xcb_id,
            type_: net_active,
            data: x11rb::protocol::xproto::ClientMessageData::from([
                2u32, 0, 0, 0, 0, // source=2 (pager), timestamp=0, currently_active=0
            ]),
        };

        self.conn.send_event(
            false,
            self.root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
        self.conn.flush().context("Failed to flush X11 connection")?;
        Ok(())
    }

    fn move_window(&mut self, xcb_id: u32, x: i32, y: i32) -> Result<()> {
        self.conn
            .configure_window(xcb_id, &ConfigureWindowAux::new().x(x).y(y))?;
        self.conn.flush().context("Failed to flush X11 connection")?;
        Ok(())
    }

    fn resize_window(&mut self, xcb_id: u32, w: u32, h: u32) -> Result<()> {
        self.conn
            .configure_window(xcb_id, &ConfigureWindowAux::new().width(w).height(h))?;
        self.conn.flush().context("Failed to flush X11 connection")?;
        Ok(())
    }

    fn close_window(&mut self, xcb_id: u32) -> Result<()> {
        // Use _NET_CLOSE_WINDOW for graceful close (respects WM protocols)
        let net_close = self
            .conn
            .intern_atom(false, b"_NET_CLOSE_WINDOW")?
            .reply()
            .context("Failed to intern _NET_CLOSE_WINDOW atom")?
            .atom;

        let event = ClientMessageEvent {
            response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
            format: 32,
            sequence: 0,
            window: xcb_id,
            type_: net_close,
            data: x11rb::protocol::xproto::ClientMessageData::from([
                0u32, 2, 0, 0, 0, // timestamp=0, source=2 (pager)
            ]),
        };

        self.conn.send_event(
            false,
            self.root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            event,
        )?;
        self.conn.flush().context("Failed to flush X11 connection")?;
        Ok(())
    }

    // Phase 4: input simulation via enigo

    fn click(&mut self, x: i32, y: i32) -> Result<()> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| anyhow::anyhow!("Mouse move failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.enigo
            .button(Button::Left, Direction::Click)
            .map_err(|e| anyhow::anyhow!("Click failed: {e}"))?;
        Ok(())
    }

    fn dblclick(&mut self, x: i32, y: i32) -> Result<()> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| anyhow::anyhow!("Mouse move failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.enigo
            .button(Button::Left, Direction::Click)
            .map_err(|e| anyhow::anyhow!("First click failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        self.enigo
            .button(Button::Left, Direction::Click)
            .map_err(|e| anyhow::anyhow!("Second click failed: {e}"))?;
        Ok(())
    }

    fn type_text(&mut self, text: &str) -> Result<()> {
        self.enigo
            .text(text)
            .map_err(|e| anyhow::anyhow!("Type failed: {e}"))?;
        Ok(())
    }

    fn press_key(&mut self, key: &str) -> Result<()> {
        let k = parse_key(key)?;
        self.enigo
            .key(k, Direction::Click)
            .map_err(|e| anyhow::anyhow!("Key press failed: {e}"))?;
        Ok(())
    }

    fn hotkey(&mut self, keys: &[String]) -> Result<()> {
        // Press all modifier keys, click the last key, release modifiers in reverse
        let parsed: Vec<Key> = keys
            .iter()
            .map(|k| parse_key(k))
            .collect::<Result<Vec<_>>>()?;

        if parsed.is_empty() {
            anyhow::bail!("No keys specified for hotkey");
        }

        let (modifiers, tail) = parsed.split_at(parsed.len() - 1);

        for m in modifiers {
            self.enigo
                .key(*m, Direction::Press)
                .map_err(|e| anyhow::anyhow!("Modifier press failed: {e}"))?;
        }

        self.enigo
            .key(tail[0], Direction::Click)
            .map_err(|e| anyhow::anyhow!("Key click failed: {e}"))?;

        for m in modifiers.iter().rev() {
            self.enigo
                .key(*m, Direction::Release)
                .map_err(|e| anyhow::anyhow!("Modifier release failed: {e}"))?;
        }

        Ok(())
    }

    fn mouse_move(&mut self, x: i32, y: i32) -> Result<()> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| anyhow::anyhow!("Mouse move failed: {e}"))?;
        Ok(())
    }

    fn scroll(&mut self, amount: i32, axis: &str) -> Result<()> {
        let ax = match axis {
            "horizontal" | "h" => Axis::Horizontal,
            _ => Axis::Vertical,
        };
        self.enigo
            .scroll(amount, ax)
            .map_err(|e| anyhow::anyhow!("Scroll failed: {e}"))?;
        Ok(())
    }

    fn drag(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> Result<()> {
        self.enigo
            .move_mouse(x1, y1, Coordinate::Abs)
            .map_err(|e| anyhow::anyhow!("Mouse move failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.enigo
            .button(Button::Left, Direction::Press)
            .map_err(|e| anyhow::anyhow!("Button press failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        self.enigo
            .move_mouse(x2, y2, Coordinate::Abs)
            .map_err(|e| anyhow::anyhow!("Mouse move to target failed: {e}"))?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.enigo
            .button(Button::Left, Direction::Release)
            .map_err(|e| anyhow::anyhow!("Button release failed: {e}"))?;
        Ok(())
    }

    // Phase 6: utility stubs

    fn screen_size(&self) -> Result<(u32, u32)> {
        anyhow::bail!("Utility commands not yet implemented (Phase 6)")
    }

    fn mouse_position(&self) -> Result<(i32, i32)> {
        anyhow::bail!("Utility commands not yet implemented (Phase 6)")
    }

    fn screenshot(&mut self, _path: &str, _annotate: bool) -> Result<String> {
        anyhow::bail!("Standalone screenshot not yet implemented (Phase 6)")
    }

    fn launch(&self, _command: &str, _args: &[String]) -> Result<u32> {
        anyhow::bail!("Launch not yet implemented (Phase 6)")
    }
}

fn parse_key(name: &str) -> Result<Key> {
    match name.to_lowercase().as_str() {
        // Modifiers
        "ctrl" | "control" => Ok(Key::Control),
        "alt" => Ok(Key::Alt),
        "shift" => Ok(Key::Shift),
        "super" | "meta" | "win" => Ok(Key::Meta),

        // Navigation / editing
        "enter" | "return" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "escape" | "esc" => Ok(Key::Escape),
        "backspace" => Ok(Key::Backspace),
        "delete" | "del" => Ok(Key::Delete),
        "space" => Ok(Key::Space),

        // Arrow keys
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),

        // Page navigation
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" => Ok(Key::PageUp),
        "pagedown" => Ok(Key::PageDown),

        // Function keys
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),

        // Single character - map to Unicode key
        s if s.len() == 1 => Ok(Key::Unicode(s.chars().next().unwrap())),

        other => anyhow::bail!("Unknown key: {other}"),
    }
}
