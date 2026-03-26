use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub screenshot: String,
    pub windows: Vec<WindowInfo>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub ref_id: String,
    pub window_id: String,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub width_mm: u32,
    pub height_mm: u32,
    pub primary: bool,
    pub automatic: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub backend: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub backend: String,
    pub display: Option<String>,
    pub session_type: Option<String>,
    pub session: String,
    pub socket_path: String,
    pub screen: ScreenSize,
    pub monitor_count: usize,
    pub monitors: Vec<MonitorInfo>,
}

impl std::fmt::Display for WindowInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = if self.focused {
            "focused"
        } else if self.minimized {
            "hidden"
        } else {
            "visible"
        };
        write!(
            f,
            "@{:<4} {:<30} ({:<7})  {},{} {}x{}",
            self.ref_id,
            truncate(&self.title, 30),
            state,
            self.x,
            self.y,
            self.width,
            self.height
        )
    }
}

#[allow(dead_code)]
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
