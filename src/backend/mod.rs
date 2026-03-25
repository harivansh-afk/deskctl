pub mod x11;

use anyhow::Result;
use crate::core::types::Snapshot;

#[allow(dead_code)]
pub trait DesktopBackend: Send {
    /// Capture a screenshot and return a z-ordered window tree with @wN refs.
    fn snapshot(&mut self, annotate: bool) -> Result<Snapshot>;

    /// Focus a window by its X11 window ID.
    fn focus_window(&mut self, xcb_id: u32) -> Result<()>;

    /// Move a window to absolute coordinates.
    fn move_window(&mut self, xcb_id: u32, x: i32, y: i32) -> Result<()>;

    /// Resize a window.
    fn resize_window(&mut self, xcb_id: u32, w: u32, h: u32) -> Result<()>;

    /// Close a window gracefully.
    fn close_window(&mut self, xcb_id: u32) -> Result<()>;

    /// Click at absolute coordinates.
    fn click(&mut self, x: i32, y: i32) -> Result<()>;

    /// Double-click at absolute coordinates.
    fn dblclick(&mut self, x: i32, y: i32) -> Result<()>;

    /// Type text into the focused window.
    fn type_text(&mut self, text: &str) -> Result<()>;

    /// Press a single key by name.
    fn press_key(&mut self, key: &str) -> Result<()>;

    /// Send a hotkey combination.
    fn hotkey(&mut self, keys: &[String]) -> Result<()>;

    /// Move the mouse cursor to absolute coordinates.
    fn mouse_move(&mut self, x: i32, y: i32) -> Result<()>;

    /// Scroll the mouse wheel.
    fn scroll(&mut self, amount: i32, axis: &str) -> Result<()>;

    /// Drag from one position to another.
    fn drag(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> Result<()>;

    /// Get the screen resolution.
    fn screen_size(&self) -> Result<(u32, u32)>;

    /// Get the current mouse position.
    fn mouse_position(&self) -> Result<(i32, i32)>;

    /// Take a screenshot and save to a path (no window tree).
    fn screenshot(&mut self, path: &str, annotate: bool) -> Result<String>;

    /// Launch an application.
    fn launch(&self, command: &str, args: &[String]) -> Result<u32>;
}
