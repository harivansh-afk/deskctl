pub mod connection;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::core::protocol::{Request, Response};

#[derive(Parser)]
#[command(
    name = "deskctl",
    bin_name = "deskctl",
    version,
    about = "Desktop control CLI for AI agents"
)]
pub struct App {
    #[command(flatten)]
    pub global: GlobalOpts,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args)]
pub struct GlobalOpts {
    /// Path to the daemon Unix socket
    #[arg(long, global = true, env = "DESKCTL_SOCKET")]
    pub socket: Option<PathBuf>,

    /// Session name (allows multiple daemon instances)
    #[arg(long, global = true, default_value = "default")]
    pub session: String,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Take a screenshot and list windows with @wN refs
    #[command(after_help = SNAPSHOT_EXAMPLES)]
    Snapshot {
        /// Draw bounding boxes and labels on the screenshot
        #[arg(long)]
        annotate: bool,
    },
    /// Click a window ref or coordinates
    #[command(after_help = CLICK_EXAMPLES)]
    Click {
        /// Selector (ref=w1, id=win1, title=Firefox, class=firefox, focused) or x,y coordinates
        selector: String,
    },
    /// Double-click a window ref or coordinates
    #[command(after_help = DBLCLICK_EXAMPLES)]
    Dblclick {
        /// Selector (ref=w1, id=win1, title=Firefox, class=firefox, focused) or x,y coordinates
        selector: String,
    },
    /// Type text into the focused window
    #[command(after_help = TYPE_EXAMPLES)]
    Type {
        /// Text to type
        text: String,
    },
    /// Press a key (e.g. enter, tab, escape)
    #[command(after_help = PRESS_EXAMPLES)]
    Press {
        /// Key name
        key: String,
    },
    /// Send a hotkey combination (e.g. ctrl c)
    #[command(after_help = HOTKEY_EXAMPLES)]
    Hotkey {
        /// Key names (e.g. ctrl shift t)
        keys: Vec<String>,
    },
    /// Mouse operations
    #[command(subcommand)]
    Mouse(MouseCmd),
    /// Focus a window by ref or name
    #[command(after_help = FOCUS_EXAMPLES)]
    Focus {
        /// Selector: ref=w1, id=win1, title=Firefox, class=firefox, focused, or a fuzzy substring
        selector: String,
    },
    /// Close a window by ref or name
    #[command(after_help = CLOSE_EXAMPLES)]
    Close {
        /// Selector: ref=w1, id=win1, title=Firefox, class=firefox, focused, or a fuzzy substring
        selector: String,
    },
    /// Move a window
    #[command(after_help = MOVE_WINDOW_EXAMPLES)]
    MoveWindow {
        /// Selector: ref=w1, id=win1, title=Firefox, class=firefox, focused, or a fuzzy substring
        selector: String,
        /// X position
        x: i32,
        /// Y position
        y: i32,
    },
    /// Resize a window
    #[command(after_help = RESIZE_WINDOW_EXAMPLES)]
    ResizeWindow {
        /// Selector: ref=w1, id=win1, title=Firefox, class=firefox, focused, or a fuzzy substring
        selector: String,
        /// Width
        w: u32,
        /// Height
        h: u32,
    },
    /// List all windows (same as snapshot but without screenshot)
    #[command(after_help = LIST_WINDOWS_EXAMPLES)]
    ListWindows,
    /// Get screen resolution
    #[command(after_help = GET_SCREEN_SIZE_EXAMPLES)]
    GetScreenSize,
    /// Get current mouse position
    #[command(after_help = GET_MOUSE_POSITION_EXAMPLES)]
    GetMousePosition,
    /// Diagnose X11 runtime, screenshot, and daemon health
    #[command(after_help = DOCTOR_EXAMPLES)]
    Doctor,
    /// Query runtime state
    #[command(subcommand)]
    Get(GetCmd),
    /// Wait for runtime state transitions
    #[command(subcommand)]
    Wait(WaitCmd),
    /// Take a screenshot without window tree
    #[command(after_help = SCREENSHOT_EXAMPLES)]
    Screenshot {
        /// Save path (default: /tmp/deskctl-{timestamp}.png)
        path: Option<PathBuf>,
        /// Draw bounding boxes and labels
        #[arg(long)]
        annotate: bool,
    },
    /// Launch an application
    #[command(after_help = LAUNCH_EXAMPLES)]
    Launch {
        /// Command to run
        command: String,
        /// Arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Daemon management (hidden - internal use)
    #[command(hide = true)]
    Daemon(DaemonCmd),
}

#[derive(Subcommand)]
pub enum MouseCmd {
    /// Move the mouse cursor
    #[command(after_help = MOUSE_MOVE_EXAMPLES)]
    Move {
        /// X coordinate
        x: i32,
        /// Y coordinate
        y: i32,
    },
    /// Scroll the mouse wheel
    #[command(after_help = MOUSE_SCROLL_EXAMPLES)]
    Scroll {
        /// Amount (positive = down, negative = up)
        amount: i32,
        /// Axis: vertical or horizontal
        #[arg(long, default_value = "vertical")]
        axis: String,
    },
    /// Drag from one position to another
    #[command(after_help = MOUSE_DRAG_EXAMPLES)]
    Drag {
        /// Start X
        x1: i32,
        /// Start Y
        y1: i32,
        /// End X
        x2: i32,
        /// End Y
        y2: i32,
    },
}

#[derive(Args)]
pub struct DaemonCmd {
    #[command(subcommand)]
    pub action: DaemonAction,
}

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Start the daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Show daemon status
    Status,
}

const GET_ACTIVE_WINDOW_EXAMPLES: &str =
    "Examples:\n  deskctl get active-window\n  deskctl --json get active-window";
const SNAPSHOT_EXAMPLES: &str =
    "Examples:\n  deskctl snapshot\n  deskctl snapshot --annotate\n  deskctl --json snapshot --annotate";
const LIST_WINDOWS_EXAMPLES: &str =
    "Examples:\n  deskctl list-windows\n  deskctl --json list-windows";
const CLICK_EXAMPLES: &str =
    "Examples:\n  deskctl click @w1\n  deskctl click 'title=Firefox'\n  deskctl click 500,300";
const DBLCLICK_EXAMPLES: &str =
    "Examples:\n  deskctl dblclick @w2\n  deskctl dblclick 'class=firefox'\n  deskctl dblclick 500,300";
const TYPE_EXAMPLES: &str =
    "Examples:\n  deskctl type \"hello world\"\n  deskctl type \"https://example.com\"";
const PRESS_EXAMPLES: &str = "Examples:\n  deskctl press enter\n  deskctl press escape";
const HOTKEY_EXAMPLES: &str = "Examples:\n  deskctl hotkey ctrl l\n  deskctl hotkey ctrl shift t";
const FOCUS_EXAMPLES: &str =
    "Examples:\n  deskctl focus @w1\n  deskctl focus 'title=Firefox'\n  deskctl focus focused";
const CLOSE_EXAMPLES: &str =
    "Examples:\n  deskctl close @w3\n  deskctl close 'id=win2'\n  deskctl close 'class=firefox'";
const MOVE_WINDOW_EXAMPLES: &str =
    "Examples:\n  deskctl move-window @w1 100 200\n  deskctl move-window 'title=Firefox' 0 0";
const RESIZE_WINDOW_EXAMPLES: &str =
    "Examples:\n  deskctl resize-window @w1 1280 720\n  deskctl resize-window 'id=win2' 800 600";
const GET_MONITORS_EXAMPLES: &str =
    "Examples:\n  deskctl get monitors\n  deskctl --json get monitors";
const GET_VERSION_EXAMPLES: &str = "Examples:\n  deskctl get version\n  deskctl --json get version";
const GET_SYSTEMINFO_EXAMPLES: &str =
    "Examples:\n  deskctl get systeminfo\n  deskctl --json get systeminfo";
const GET_SCREEN_SIZE_EXAMPLES: &str =
    "Examples:\n  deskctl get-screen-size\n  deskctl --json get-screen-size";
const GET_MOUSE_POSITION_EXAMPLES: &str =
    "Examples:\n  deskctl get-mouse-position\n  deskctl --json get-mouse-position";
const DOCTOR_EXAMPLES: &str = "Examples:\n  deskctl doctor\n  deskctl --json doctor";
const WAIT_WINDOW_EXAMPLES: &str = "Examples:\n  deskctl wait window --selector 'title=Firefox' --timeout 10\n  deskctl --json wait window --selector 'class=firefox' --poll-ms 100";
const WAIT_FOCUS_EXAMPLES: &str = "Examples:\n  deskctl wait focus --selector 'id=win3' --timeout 5\n  deskctl wait focus --selector focused --poll-ms 200";
const SCREENSHOT_EXAMPLES: &str =
    "Examples:\n  deskctl screenshot\n  deskctl screenshot /tmp/screen.png\n  deskctl screenshot --annotate";
const LAUNCH_EXAMPLES: &str =
    "Examples:\n  deskctl launch firefox\n  deskctl launch code -- --new-window";
const MOUSE_MOVE_EXAMPLES: &str =
    "Examples:\n  deskctl mouse move 500 300\n  deskctl mouse move 0 0";
const MOUSE_SCROLL_EXAMPLES: &str =
    "Examples:\n  deskctl mouse scroll 3\n  deskctl mouse scroll -3 --axis vertical";
const MOUSE_DRAG_EXAMPLES: &str = "Examples:\n  deskctl mouse drag 100 100 500 500";

#[derive(Subcommand)]
pub enum GetCmd {
    /// Show the currently focused window
    #[command(after_help = GET_ACTIVE_WINDOW_EXAMPLES)]
    ActiveWindow,
    /// List current monitor geometry and metadata
    #[command(after_help = GET_MONITORS_EXAMPLES)]
    Monitors,
    /// Show deskctl version and backend information
    #[command(after_help = GET_VERSION_EXAMPLES)]
    Version,
    /// Show runtime-focused diagnostic information
    #[command(after_help = GET_SYSTEMINFO_EXAMPLES)]
    Systeminfo,
}

#[derive(Subcommand)]
pub enum WaitCmd {
    /// Wait until a window matching the selector exists
    #[command(after_help = WAIT_WINDOW_EXAMPLES)]
    Window(WaitSelectorOpts),
    /// Wait until the selector resolves to a focused window
    #[command(after_help = WAIT_FOCUS_EXAMPLES)]
    Focus(WaitSelectorOpts),
}

#[derive(Args)]
pub struct WaitSelectorOpts {
    /// Selector: ref=w1, id=win1, title=Firefox, class=firefox, focused, or a fuzzy substring
    #[arg(long)]
    pub selector: String,

    /// Timeout in seconds
    #[arg(long, default_value_t = 10)]
    pub timeout: u64,

    /// Poll interval in milliseconds
    #[arg(long = "poll-ms", default_value_t = 250)]
    pub poll_ms: u64,
}

pub fn run() -> Result<()> {
    let app = App::parse();

    // Handle daemon subcommands that don't need a running daemon
    if let Command::Daemon(ref cmd) = app.command {
        return match cmd.action {
            DaemonAction::Start => connection::start_daemon(&app.global),
            DaemonAction::Stop => connection::stop_daemon(&app.global),
            DaemonAction::Status => connection::daemon_status(&app.global),
        };
    }

    if let Command::Doctor = app.command {
        return connection::run_doctor(&app.global);
    }

    // All other commands need a daemon connection
    let request = build_request(&app.command)?;
    let response = connection::send_command(&app.global, &request)?;
    let success = response.success;

    if app.global.json {
        println!("{}", serde_json::to_string_pretty(&response)?);
        if !success {
            std::process::exit(1);
        }
    } else {
        print_response(&app.command, &response)?;
    }

    Ok(())
}

fn build_request(cmd: &Command) -> Result<Request> {
    use serde_json::json;
    let req = match cmd {
        Command::Snapshot { annotate } => {
            Request::new("snapshot").with_extra("annotate", json!(annotate))
        }
        Command::Click { selector } => {
            Request::new("click").with_extra("selector", json!(selector))
        }
        Command::Dblclick { selector } => {
            Request::new("dblclick").with_extra("selector", json!(selector))
        }
        Command::Type { text } => Request::new("type").with_extra("text", json!(text)),
        Command::Press { key } => Request::new("press").with_extra("key", json!(key)),
        Command::Hotkey { keys } => Request::new("hotkey").with_extra("keys", json!(keys)),
        Command::Mouse(sub) => match sub {
            MouseCmd::Move { x, y } => Request::new("mouse-move")
                .with_extra("x", json!(x))
                .with_extra("y", json!(y)),
            MouseCmd::Scroll { amount, axis } => Request::new("mouse-scroll")
                .with_extra("amount", json!(amount))
                .with_extra("axis", json!(axis)),
            MouseCmd::Drag { x1, y1, x2, y2 } => Request::new("mouse-drag")
                .with_extra("x1", json!(x1))
                .with_extra("y1", json!(y1))
                .with_extra("x2", json!(x2))
                .with_extra("y2", json!(y2)),
        },
        Command::Focus { selector } => {
            Request::new("focus").with_extra("selector", json!(selector))
        }
        Command::Close { selector } => {
            Request::new("close").with_extra("selector", json!(selector))
        }
        Command::MoveWindow { selector, x, y } => Request::new("move-window")
            .with_extra("selector", json!(selector))
            .with_extra("x", json!(x))
            .with_extra("y", json!(y)),
        Command::ResizeWindow { selector, w, h } => Request::new("resize-window")
            .with_extra("selector", json!(selector))
            .with_extra("w", json!(w))
            .with_extra("h", json!(h)),
        Command::ListWindows => Request::new("list-windows"),
        Command::GetScreenSize => Request::new("get-screen-size"),
        Command::GetMousePosition => Request::new("get-mouse-position"),
        Command::Doctor => unreachable!(),
        Command::Get(sub) => match sub {
            GetCmd::ActiveWindow => Request::new("get-active-window"),
            GetCmd::Monitors => Request::new("get-monitors"),
            GetCmd::Version => Request::new("get-version"),
            GetCmd::Systeminfo => Request::new("get-systeminfo"),
        },
        Command::Wait(sub) => match sub {
            WaitCmd::Window(opts) => Request::new("wait-window")
                .with_extra("selector", json!(opts.selector))
                .with_extra("timeout_ms", json!(opts.timeout * 1000))
                .with_extra("poll_ms", json!(opts.poll_ms)),
            WaitCmd::Focus(opts) => Request::new("wait-focus")
                .with_extra("selector", json!(opts.selector))
                .with_extra("timeout_ms", json!(opts.timeout * 1000))
                .with_extra("poll_ms", json!(opts.poll_ms)),
        },
        Command::Screenshot { path, annotate } => {
            let mut req = Request::new("screenshot").with_extra("annotate", json!(annotate));
            if let Some(p) = path {
                req = req.with_extra("path", json!(p.to_string_lossy()));
            }
            req
        }
        Command::Launch { command, args } => Request::new("launch")
            .with_extra("command", json!(command))
            .with_extra("args", json!(args)),
        Command::Daemon(_) => unreachable!(),
    };
    Ok(req)
}

fn print_response(cmd: &Command, response: &Response) -> Result<()> {
    if !response.success {
        for line in render_error_lines(response) {
            eprintln!("{line}");
        }
        std::process::exit(1);
    }
    for line in render_success_lines(cmd, response.data.as_ref())? {
        println!("{line}");
    }
    Ok(())
}

fn render_success_lines(cmd: &Command, data: Option<&serde_json::Value>) -> Result<Vec<String>> {
    let Some(data) = data else {
        return Ok(vec!["ok".to_string()]);
    };

    let lines = match cmd {
        Command::Snapshot { .. } | Command::ListWindows => render_window_listing(data),
        Command::Get(GetCmd::ActiveWindow)
        | Command::Wait(WaitCmd::Window(_))
        | Command::Wait(WaitCmd::Focus(_)) => render_window_wait_or_read(data),
        Command::Get(GetCmd::Monitors) => render_monitor_listing(data),
        Command::Get(GetCmd::Version) => vec![render_version_line(data)],
        Command::Get(GetCmd::Systeminfo) => render_systeminfo_lines(data),
        Command::GetScreenSize => vec![render_screen_size_line(data)],
        Command::GetMousePosition => vec![render_mouse_position_line(data)],
        Command::Screenshot { annotate, .. } => render_screenshot_lines(data, *annotate),
        Command::Click { .. } => vec![render_click_line(data, false)],
        Command::Dblclick { .. } => vec![render_click_line(data, true)],
        Command::Type { .. } => vec![render_type_line(data)],
        Command::Press { .. } => vec![render_press_line(data)],
        Command::Hotkey { .. } => vec![render_hotkey_line(data)],
        Command::Mouse(sub) => vec![render_mouse_line(sub, data)],
        Command::Focus { .. } => vec![render_window_action_line("Focused", data)],
        Command::Close { .. } => vec![render_window_action_line("Closed", data)],
        Command::MoveWindow { .. } => vec![render_move_window_line(data)],
        Command::ResizeWindow { .. } => vec![render_resize_window_line(data)],
        Command::Launch { .. } => vec![render_launch_line(data)],
        Command::Doctor | Command::Daemon(_) => vec![serde_json::to_string_pretty(data)?],
    };

    Ok(lines)
}

fn render_error_lines(response: &Response) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(err) = &response.error {
        lines.push(format!("Error: {err}"));
    }

    let Some(data) = response.data.as_ref() else {
        return lines;
    };

    let Some(kind) = data.get("kind").and_then(|value| value.as_str()) else {
        return lines;
    };

    match kind {
        "selector_not_found" => {
            let selector = data
                .get("selector")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            let mode = data
                .get("mode")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            lines.push(format!("Selector: {selector} (mode: {mode})"));
        }
        "selector_invalid" => {
            let selector = data
                .get("selector")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            let mode = data
                .get("mode")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            lines.push(format!("Selector: {selector} (mode: {mode})"));
            if let Some(message) = data.get("message").and_then(|value| value.as_str()) {
                lines.push(format!("Reason: {message}"));
            }
        }
        "selector_ambiguous" => {
            let selector = data
                .get("selector")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            let mode = data
                .get("mode")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            lines.push(format!("Selector: {selector} (mode: {mode})"));
            if let Some(candidates) = data.get("candidates").and_then(|value| value.as_array()) {
                lines.push("Candidates:".to_string());
                for candidate in candidates {
                    lines.push(window_line(candidate));
                }
            }
        }
        "timeout" => {
            let selector = data
                .get("selector")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            let wait = data
                .get("wait")
                .and_then(|value| value.as_str())
                .unwrap_or("wait");
            let timeout_ms = data
                .get("timeout_ms")
                .and_then(|value| value.as_u64())
                .unwrap_or(0);
            lines.push(format!(
                "Timed out after {timeout_ms}ms waiting for {wait} selector {selector}"
            ));
            if let Some(observation) = data.get("last_observation") {
                lines.extend(render_last_observation_lines(observation));
            }
        }
        "not_found" => {
            if data
                .get("mode")
                .and_then(|value| value.as_str())
                .is_some_and(|mode| mode == "focused")
            {
                lines.push("No focused window is available.".to_string());
            }
        }
        _ => {}
    }

    lines
}

fn render_last_observation_lines(observation: &serde_json::Value) -> Vec<String> {
    let mut lines = Vec::new();
    let Some(kind) = observation.get("kind").and_then(|value| value.as_str()) else {
        return lines;
    };

    match kind {
        "window_not_focused" => {
            lines.push(
                "Last observation: matching window exists but is not focused yet.".to_string(),
            );
            if let Some(window) = observation.get("window") {
                lines.push(window_line(window));
            }
        }
        "selector_not_found" => {
            let selector = observation
                .get("selector")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            let mode = observation
                .get("mode")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            lines.push(format!(
                "Last observation: no window matched selector {selector} (mode: {mode})"
            ));
        }
        _ => {
            lines.push(format!(
                "Last observation: {}",
                serde_json::to_string(observation).unwrap_or_else(|_| kind.to_string())
            ));
        }
    }

    lines
}

fn render_window_listing(data: &serde_json::Value) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(screenshot) = data.get("screenshot").and_then(|value| value.as_str()) {
        lines.push(format!("Screenshot: {screenshot}"));
    }
    if let Some(windows) = data.get("windows").and_then(|value| value.as_array()) {
        lines.push(format!("Windows: {}", windows.len()));
        for window in windows {
            lines.push(window_line(window));
        }
    }
    lines
}

fn render_window_wait_or_read(data: &serde_json::Value) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(window) = data.get("window") {
        lines.push(window_line(window));
    }
    if let Some(elapsed_ms) = data.get("elapsed_ms").and_then(|value| value.as_u64()) {
        lines.push(format!("Elapsed: {elapsed_ms}ms"));
    }
    lines
}

fn render_monitor_listing(data: &serde_json::Value) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(count) = data.get("count").and_then(|value| value.as_u64()) {
        lines.push(format!("Monitors: {count}"));
    }
    if let Some(monitors) = data.get("monitors").and_then(|value| value.as_array()) {
        for monitor in monitors {
            let name = monitor
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("monitor");
            let x = monitor
                .get("x")
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            let y = monitor
                .get("y")
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            let width = monitor
                .get("width")
                .and_then(|value| value.as_u64())
                .unwrap_or(0);
            let height = monitor
                .get("height")
                .and_then(|value| value.as_u64())
                .unwrap_or(0);
            let primary = monitor
                .get("primary")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            let automatic = monitor
                .get("automatic")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            let mut flags = Vec::new();
            if primary {
                flags.push("primary");
            }
            if automatic {
                flags.push("automatic");
            }
            let suffix = if flags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", flags.join(", "))
            };
            lines.push(format!("{name:<16} {x},{y} {width}x{height}{suffix}"));
        }
    }
    lines
}

fn render_version_line(data: &serde_json::Value) -> String {
    let version = data
        .get("version")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let backend = data
        .get("backend")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    format!("deskctl {version} ({backend})")
}

fn render_systeminfo_lines(data: &serde_json::Value) -> Vec<String> {
    let mut lines = Vec::new();
    let backend = data
        .get("backend")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    lines.push(format!("Backend: {backend}"));
    if let Some(display) = data.get("display").and_then(|value| value.as_str()) {
        lines.push(format!("Display: {display}"));
    }
    if let Some(session_type) = data.get("session_type").and_then(|value| value.as_str()) {
        lines.push(format!("Session type: {session_type}"));
    }
    if let Some(session) = data.get("session").and_then(|value| value.as_str()) {
        lines.push(format!("Session: {session}"));
    }
    if let Some(socket_path) = data.get("socket_path").and_then(|value| value.as_str()) {
        lines.push(format!("Socket: {socket_path}"));
    }
    if let Some(screen) = data.get("screen") {
        lines.push(format!("Screen: {}", screen_dimensions(screen)));
    }
    if let Some(count) = data.get("monitor_count").and_then(|value| value.as_u64()) {
        lines.push(format!("Monitor count: {count}"));
    }
    if let Some(monitors) = data.get("monitors").and_then(|value| value.as_array()) {
        for monitor in monitors {
            lines.push(format!(
                "  {}",
                render_monitor_listing(&serde_json::json!({"monitors": [monitor]}))[0]
            ));
        }
    }
    lines
}

fn render_screen_size_line(data: &serde_json::Value) -> String {
    format!("Screen: {}", screen_dimensions(data))
}

fn render_mouse_position_line(data: &serde_json::Value) -> String {
    let x = data.get("x").and_then(|value| value.as_i64()).unwrap_or(0);
    let y = data.get("y").and_then(|value| value.as_i64()).unwrap_or(0);
    format!("Pointer: {x},{y}")
}

fn render_screenshot_lines(data: &serde_json::Value, annotate: bool) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(screenshot) = data.get("screenshot").and_then(|value| value.as_str()) {
        lines.push(format!("Screenshot: {screenshot}"));
    }
    if annotate {
        if let Some(windows) = data.get("windows").and_then(|value| value.as_array()) {
            lines.push(format!("Annotated windows: {}", windows.len()));
            for window in windows {
                lines.push(window_line(window));
            }
        }
    }
    lines
}

fn render_click_line(data: &serde_json::Value, double: bool) -> String {
    let action = if double { "Double-clicked" } else { "Clicked" };
    let key = if double { "double_clicked" } else { "clicked" };
    let x = data
        .get(key)
        .and_then(|value| value.get("x"))
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let y = data
        .get(key)
        .and_then(|value| value.get("y"))
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    match target_summary(data) {
        Some(target) => format!("{action} {x},{y} on {target}"),
        None => format!("{action} {x},{y}"),
    }
}

fn render_type_line(data: &serde_json::Value) -> String {
    let typed = data
        .get("typed")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    format!("Typed: {}", quoted_summary(typed, 60))
}

fn render_press_line(data: &serde_json::Value) -> String {
    let key = data
        .get("pressed")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    format!("Pressed: {key}")
}

fn render_hotkey_line(data: &serde_json::Value) -> String {
    let keys = data
        .get("hotkey")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|value| value.as_str())
                .collect::<Vec<_>>()
                .join("+")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    format!("Hotkey: {keys}")
}

fn render_mouse_line(sub: &MouseCmd, data: &serde_json::Value) -> String {
    match sub {
        MouseCmd::Move { .. } => {
            let x = data
                .get("moved")
                .and_then(|value| value.get("x"))
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            let y = data
                .get("moved")
                .and_then(|value| value.get("y"))
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            format!("Moved pointer to {x},{y}")
        }
        MouseCmd::Scroll { .. } => {
            let amount = data
                .get("scrolled")
                .and_then(|value| value.get("amount"))
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            let axis = data
                .get("scrolled")
                .and_then(|value| value.get("axis"))
                .and_then(|value| value.as_str())
                .unwrap_or("vertical");
            format!("Scrolled {axis} by {amount}")
        }
        MouseCmd::Drag { .. } => {
            let x1 = data
                .get("dragged")
                .and_then(|value| value.get("from"))
                .and_then(|value| value.get("x"))
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            let y1 = data
                .get("dragged")
                .and_then(|value| value.get("from"))
                .and_then(|value| value.get("y"))
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            let x2 = data
                .get("dragged")
                .and_then(|value| value.get("to"))
                .and_then(|value| value.get("x"))
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            let y2 = data
                .get("dragged")
                .and_then(|value| value.get("to"))
                .and_then(|value| value.get("y"))
                .and_then(|value| value.as_i64())
                .unwrap_or(0);
            format!("Dragged {x1},{y1} -> {x2},{y2}")
        }
    }
}

fn render_window_action_line(action: &str, data: &serde_json::Value) -> String {
    match target_summary(data) {
        Some(target) => format!("{action} {target}"),
        None => action.to_string(),
    }
}

fn render_move_window_line(data: &serde_json::Value) -> String {
    let x = data.get("x").and_then(|value| value.as_i64()).unwrap_or(0);
    let y = data.get("y").and_then(|value| value.as_i64()).unwrap_or(0);
    match target_summary(data) {
        Some(target) => format!("Moved {target} to {x},{y}"),
        None => format!("Moved window to {x},{y}"),
    }
}

fn render_resize_window_line(data: &serde_json::Value) -> String {
    let width = data
        .get("width")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let height = data
        .get("height")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    match target_summary(data) {
        Some(target) => format!("Resized {target} to {width}x{height}"),
        None => format!("Resized window to {width}x{height}"),
    }
}

fn render_launch_line(data: &serde_json::Value) -> String {
    let command = data
        .get("command")
        .and_then(|value| value.as_str())
        .unwrap_or("command");
    let pid = data
        .get("pid")
        .and_then(|value| value.as_u64())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    format!("Launched {command} (pid {pid})")
}

fn window_line(window: &serde_json::Value) -> String {
    let ref_id = window
        .get("ref_id")
        .and_then(|value| value.as_str())
        .unwrap_or("?");
    let window_id = window
        .get("window_id")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown");
    let title = window
        .get("title")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let focused = window
        .get("focused")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let minimized = window
        .get("minimized")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let x = window
        .get("x")
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let y = window
        .get("y")
        .and_then(|value| value.as_i64())
        .unwrap_or(0);
    let width = window
        .get("width")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let height = window
        .get("height")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let state = if focused {
        "focused"
    } else if minimized {
        "hidden"
    } else {
        "visible"
    };
    format!(
        "@{ref_id:<4} {:<30} ({state:<7})  {x},{y} {width}x{height} [{window_id}]",
        truncate_display(title, 30)
    )
}

fn target_summary(data: &serde_json::Value) -> Option<String> {
    let ref_id = data.get("ref_id").and_then(|value| value.as_str());
    let window_id = data.get("window_id").and_then(|value| value.as_str());
    let title = data
        .get("title")
        .or_else(|| data.get("window"))
        .and_then(|value| value.as_str());

    match (ref_id, window_id, title) {
        (Some(ref_id), Some(window_id), Some(title)) => Some(format!(
            "@{ref_id} [{window_id}] {}",
            quoted_summary(title, 40)
        )),
        (None, Some(window_id), Some(title)) => {
            Some(format!("[{window_id}] {}", quoted_summary(title, 40)))
        }
        (Some(ref_id), Some(window_id), None) => Some(format!("@{ref_id} [{window_id}]")),
        (None, Some(window_id), None) => Some(format!("[{window_id}]")),
        _ => None,
    }
}

fn quoted_summary(value: &str, max_chars: usize) -> String {
    format!("\"{}\"", truncate_display(value, max_chars))
}

fn screen_dimensions(data: &serde_json::Value) -> String {
    let width = data
        .get("width")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let height = data
        .get("height")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    format!("{width}x{height}")
}

fn truncate_display(value: &str, max_chars: usize) -> String {
    let char_count = value.chars().count();
    if char_count <= max_chars {
        return value.to_string();
    }

    let truncated: String = value.chars().take(max_chars.saturating_sub(3)).collect();
    format!("{truncated}...")
}

#[cfg(test)]
mod tests {
    use super::{
        render_error_lines, render_screen_size_line, render_success_lines, target_summary,
        truncate_display, App, Command, Response,
    };
    use clap::CommandFactory;
    use serde_json::json;

    #[test]
    fn help_examples_include_snapshot_examples() {
        let help = App::command()
            .find_subcommand_mut("snapshot")
            .expect("snapshot subcommand must exist")
            .render_long_help()
            .to_string();
        assert!(help.contains("deskctl snapshot --annotate"));
    }

    #[test]
    fn root_help_uses_public_bin_name() {
        let help = App::command().render_help().to_string();
        assert!(help.contains("Usage: deskctl [OPTIONS] <COMMAND>"));
    }

    #[test]
    fn window_listing_text_includes_window_ids() {
        let lines = render_success_lines(
            &Command::ListWindows,
            Some(&json!({
                "windows": [{
                    "ref_id": "w1",
                    "window_id": "win1",
                    "title": "Firefox",
                    "app_name": "firefox",
                    "x": 0,
                    "y": 0,
                    "width": 1280,
                    "height": 720,
                    "focused": true,
                    "minimized": false
                }]
            })),
        )
        .unwrap();

        assert_eq!(lines[0], "Windows: 1");
        assert!(lines[1].contains("[win1]"));
        assert!(lines[1].contains("@w1"));
    }

    #[test]
    fn action_text_includes_target_identity() {
        let lines = render_success_lines(
            &Command::Focus {
                selector: "title=Firefox".to_string(),
            },
            Some(&json!({
                "action": "focus",
                "window": "Firefox",
                "title": "Firefox",
                "ref_id": "w2",
                "window_id": "win7"
            })),
        )
        .unwrap();

        assert_eq!(lines, vec!["Focused @w2 [win7] \"Firefox\""]);
    }

    #[test]
    fn timeout_errors_render_last_observation() {
        let lines = render_error_lines(&Response::err_with_data(
            "Timed out waiting for focus to match selector: title=Firefox",
            json!({
                "kind": "timeout",
                "wait": "focus",
                "selector": "title=Firefox",
                "timeout_ms": 1000,
                "last_observation": {
                    "kind": "window_not_focused",
                    "window": {
                        "ref_id": "w1",
                        "window_id": "win1",
                        "title": "Firefox",
                        "app_name": "firefox",
                        "x": 0,
                        "y": 0,
                        "width": 1280,
                        "height": 720,
                        "focused": false,
                        "minimized": false
                    }
                }
            }),
        ));

        assert!(lines
            .iter()
            .any(|line| line
                .contains("Timed out after 1000ms waiting for focus selector title=Firefox")));
        assert!(lines
            .iter()
            .any(|line| line.contains("matching window exists but is not focused yet")));
        assert!(lines.iter().any(|line| line.contains("[win1]")));
    }

    #[test]
    fn screen_size_text_is_compact() {
        assert_eq!(
            render_screen_size_line(&json!({"width": 1440, "height": 900})),
            "Screen: 1440x900"
        );
    }

    #[test]
    fn target_summary_prefers_ref_and_window_id() {
        let summary = target_summary(&json!({
            "ref_id": "w1",
            "window_id": "win1",
            "title": "Firefox"
        }));
        assert_eq!(summary.as_deref(), Some("@w1 [win1] \"Firefox\""));
    }

    #[test]
    fn truncate_display_is_char_safe() {
        let input = format!("fire{}fox", '\u{00E9}');
        assert_eq!(truncate_display(&input, 7), "fire...");
    }
}
