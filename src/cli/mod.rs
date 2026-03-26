pub mod connection;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::core::protocol::{Request, Response};

#[derive(Parser)]
#[command(name = "deskctl", version, about = "Desktop control CLI for AI agents")]
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
    Snapshot {
        /// Draw bounding boxes and labels on the screenshot
        #[arg(long)]
        annotate: bool,
    },
    /// Click a window ref or coordinates
    Click {
        /// @w1 or x,y coordinates
        selector: String,
    },
    /// Double-click a window ref or coordinates
    Dblclick {
        /// @w1 or x,y coordinates
        selector: String,
    },
    /// Type text into the focused window
    Type {
        /// Text to type
        text: String,
    },
    /// Press a key (e.g. enter, tab, escape)
    Press {
        /// Key name
        key: String,
    },
    /// Send a hotkey combination (e.g. ctrl c)
    Hotkey {
        /// Key names (e.g. ctrl shift t)
        keys: Vec<String>,
    },
    /// Mouse operations
    #[command(subcommand)]
    Mouse(MouseCmd),
    /// Focus a window by ref or name
    Focus {
        /// @w1 or window name substring
        selector: String,
    },
    /// Close a window by ref or name
    Close {
        /// @w1 or window name substring
        selector: String,
    },
    /// Move a window
    MoveWindow {
        /// @w1 or window name substring
        selector: String,
        /// X position
        x: i32,
        /// Y position
        y: i32,
    },
    /// Resize a window
    ResizeWindow {
        /// @w1 or window name substring
        selector: String,
        /// Width
        w: u32,
        /// Height
        h: u32,
    },
    /// List all windows (same as snapshot but without screenshot)
    ListWindows,
    /// Get screen resolution
    GetScreenSize,
    /// Get current mouse position
    GetMousePosition,
    /// Diagnose X11 runtime, screenshot, and daemon health
    Doctor,
    /// Query runtime state
    #[command(subcommand)]
    Get(GetCmd),
    /// Wait for runtime state transitions
    #[command(subcommand)]
    Wait(WaitCmd),
    /// Take a screenshot without window tree
    Screenshot {
        /// Save path (default: /tmp/deskctl-{timestamp}.png)
        path: Option<PathBuf>,
        /// Draw bounding boxes and labels
        #[arg(long)]
        annotate: bool,
    },
    /// Launch an application
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
    Move {
        /// X coordinate
        x: i32,
        /// Y coordinate
        y: i32,
    },
    /// Scroll the mouse wheel
    Scroll {
        /// Amount (positive = down, negative = up)
        amount: i32,
        /// Axis: vertical or horizontal
        #[arg(long, default_value = "vertical")]
        axis: String,
    },
    /// Drag from one position to another
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
const GET_MONITORS_EXAMPLES: &str =
    "Examples:\n  deskctl get monitors\n  deskctl --json get monitors";
const GET_VERSION_EXAMPLES: &str = "Examples:\n  deskctl get version\n  deskctl --json get version";
const GET_SYSTEMINFO_EXAMPLES: &str =
    "Examples:\n  deskctl get systeminfo\n  deskctl --json get systeminfo";
const WAIT_WINDOW_EXAMPLES: &str = "Examples:\n  deskctl wait window --selector 'title=Firefox' --timeout 10\n  deskctl --json wait window --selector 'class=firefox' --poll-ms 100";
const WAIT_FOCUS_EXAMPLES: &str = "Examples:\n  deskctl wait focus --selector 'id=win3' --timeout 5\n  deskctl wait focus --selector focused --poll-ms 200";

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

    if app.global.json {
        println!("{}", serde_json::to_string_pretty(&response)?);
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
        if let Some(ref err) = response.error {
            eprintln!("Error: {err}");
        }
        if let Some(ref data) = response.data {
            if let Some(kind) = data.get("kind").and_then(|value| value.as_str()) {
                match kind {
                    "selector_ambiguous" => {
                        if let Some(candidates) = data.get("candidates").and_then(|v| v.as_array())
                        {
                            eprintln!("Candidates:");
                            for candidate in candidates {
                                print_window_to_stderr(candidate);
                            }
                        }
                    }
                    "timeout" => {
                        if let Some(selector) = data.get("selector").and_then(|v| v.as_str()) {
                            let wait = data.get("wait").and_then(|v| v.as_str()).unwrap_or("wait");
                            let timeout_ms =
                                data.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(0);
                            eprintln!(
                                "Timed out after {timeout_ms}ms waiting for {wait} selector {selector}"
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
        std::process::exit(1);
    }
    if let Some(ref data) = response.data {
        // For snapshot, print compact text format
        if matches!(cmd, Command::Snapshot { .. } | Command::ListWindows) {
            if let Some(screenshot) = data.get("screenshot").and_then(|v| v.as_str()) {
                println!("Screenshot: {screenshot}");
            }
            if let Some(windows) = data.get("windows").and_then(|v| v.as_array()) {
                println!("Windows:");
                for w in windows {
                    let ref_id = w.get("ref_id").and_then(|v| v.as_str()).unwrap_or("?");
                    let title = w.get("title").and_then(|v| v.as_str()).unwrap_or("");
                    let focused = w.get("focused").and_then(|v| v.as_bool()).unwrap_or(false);
                    let minimized = w
                        .get("minimized")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let x = w.get("x").and_then(|v| v.as_i64()).unwrap_or(0);
                    let y = w.get("y").and_then(|v| v.as_i64()).unwrap_or(0);
                    let width = w.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
                    let height = w.get("height").and_then(|v| v.as_u64()).unwrap_or(0);
                    let state = if focused {
                        "focused"
                    } else if minimized {
                        "hidden"
                    } else {
                        "visible"
                    };
                    let display_title = if title.len() > 30 {
                        format!("{}...", &title[..27])
                    } else {
                        title.to_string()
                    };
                    println!(
                        "@{:<4} {:<30} ({:<7})  {},{} {}x{}",
                        ref_id, display_title, state, x, y, width, height
                    );
                }
            }
        } else if matches!(
            cmd,
            Command::Get(GetCmd::ActiveWindow)
                | Command::Wait(WaitCmd::Window(_))
                | Command::Wait(WaitCmd::Focus(_))
        ) {
            if let Some(window) = data.get("window") {
                print_window(window);
                if let Some(elapsed_ms) = data.get("elapsed_ms").and_then(|v| v.as_u64()) {
                    println!("Elapsed: {elapsed_ms}ms");
                }
            } else {
                println!("{}", serde_json::to_string_pretty(data)?);
            }
        } else if matches!(cmd, Command::Get(GetCmd::Monitors)) {
            if let Some(monitors) = data.get("monitors").and_then(|v| v.as_array()) {
                for monitor in monitors {
                    let name = monitor
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("monitor");
                    let x = monitor.get("x").and_then(|v| v.as_i64()).unwrap_or(0);
                    let y = monitor.get("y").and_then(|v| v.as_i64()).unwrap_or(0);
                    let width = monitor.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
                    let height = monitor.get("height").and_then(|v| v.as_u64()).unwrap_or(0);
                    let primary = monitor
                        .get("primary")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let primary_suffix = if primary { " primary" } else { "" };
                    println!(
                        "{name:<16} {},{} {}x{}{primary_suffix}",
                        x, y, width, height
                    );
                }
            }
        } else if matches!(cmd, Command::Get(GetCmd::Version)) {
            let version = data
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let backend = data
                .get("backend")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            println!("deskctl {version} ({backend})");
        } else if matches!(cmd, Command::Get(GetCmd::Systeminfo)) {
            println!("{}", serde_json::to_string_pretty(data)?);
        } else {
            // Generic: print JSON data
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }
    Ok(())
}

fn print_window(window: &serde_json::Value) {
    print_window_line(window, false);
}

fn print_window_to_stderr(window: &serde_json::Value) {
    print_window_line(window, true);
}

fn print_window_line(window: &serde_json::Value, stderr: bool) {
    let ref_id = window.get("ref_id").and_then(|v| v.as_str()).unwrap_or("?");
    let window_id = window
        .get("window_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let title = window.get("title").and_then(|v| v.as_str()).unwrap_or("");
    let focused = window
        .get("focused")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let minimized = window
        .get("minimized")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let x = window.get("x").and_then(|v| v.as_i64()).unwrap_or(0);
    let y = window.get("y").and_then(|v| v.as_i64()).unwrap_or(0);
    let width = window.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
    let height = window.get("height").and_then(|v| v.as_u64()).unwrap_or(0);
    let state = if focused {
        "focused"
    } else if minimized {
        "hidden"
    } else {
        "visible"
    };
    let line = format!(
        "@{:<4} {:<30} ({:<7})  {},{} {}x{} [{}]",
        ref_id,
        if title.len() > 30 {
            format!("{}...", &title[..27])
        } else {
            title.to_string()
        },
        state,
        x,
        y,
        width,
        height,
        window_id
    );
    if stderr {
        eprintln!("{line}");
    } else {
        println!("{line}");
    }
}
