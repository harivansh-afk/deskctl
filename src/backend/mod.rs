pub mod annotate;
pub mod x11;

use anyhow::Result;
use image::RgbaImage;

#[derive(Debug, Clone)]
pub struct BackendWindow {
    pub native_id: u32,
    pub title: String,
    pub app_name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub focused: bool,
    pub minimized: bool,
}

#[allow(dead_code)]
pub trait DesktopBackend: Send {
    /// Collect z-ordered windows for read-only queries and targeting.
    fn list_windows(&mut self) -> Result<Vec<BackendWindow>>;

    /// Capture the current desktop image without writing it to disk.
    fn capture_screenshot(&mut self) -> Result<RgbaImage>;

    /// Focus a window by its backend-native window handle.
    fn focus_window(&mut self, native_id: u32) -> Result<()>;

    /// Move a window to absolute coordinates.
    fn move_window(&mut self, native_id: u32, x: i32, y: i32) -> Result<()>;

    /// Resize a window.
    fn resize_window(&mut self, native_id: u32, w: u32, h: u32) -> Result<()>;

    /// Close a window gracefully.
    fn close_window(&mut self, native_id: u32) -> Result<()>;

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

    /// Launch an application.
    fn launch(&self, command: &str, args: &[String]) -> Result<u32>;
}
