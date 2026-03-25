mod connection;

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;

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
            Request::new("snapshot")
                .with_extra("annotate", json!(annotate))
        }
        Command::Click { selector } => {
            Request::new("click")
                .with_extra("selector", json!(selector))
        }
        Command::Dblclick { selector } => {
            Request::new("dblclick")
                .with_extra("selector", json!(selector))
        }
        Command::Type { text } => {
            Request::new("type")
                .with_extra("text", json!(text))
        }
        Command::Press { key } => {
            Request::new("press")
                .with_extra("key", json!(key))
        }
        Command::Hotkey { keys } => {
            Request::new("hotkey")
                .with_extra("keys", json!(keys))
        }
        Command::Mouse(sub) => match sub {
            MouseCmd::Move { x, y } => {
                Request::new("mouse-move")
                    .with_extra("x", json!(x))
                    .with_extra("y", json!(y))
            }
            MouseCmd::Scroll { amount, axis } => {
                Request::new("mouse-scroll")
                    .with_extra("amount", json!(amount))
                    .with_extra("axis", json!(axis))
            }
            MouseCmd::Drag { x1, y1, x2, y2 } => {
                Request::new("mouse-drag")
                    .with_extra("x1", json!(x1))
                    .with_extra("y1", json!(y1))
                    .with_extra("x2", json!(x2))
                    .with_extra("y2", json!(y2))
            }
        },
        Command::Focus { selector } => {
            Request::new("focus")
                .with_extra("selector", json!(selector))
        }
        Command::Close { selector } => {
            Request::new("close")
                .with_extra("selector", json!(selector))
        }
        Command::MoveWindow { selector, x, y } => {
            Request::new("move-window")
                .with_extra("selector", json!(selector))
                .with_extra("x", json!(x))
                .with_extra("y", json!(y))
        }
        Command::ResizeWindow { selector, w, h } => {
            Request::new("resize-window")
                .with_extra("selector", json!(selector))
                .with_extra("w", json!(w))
                .with_extra("h", json!(h))
        }
        Command::ListWindows => Request::new("list-windows"),
        Command::GetScreenSize => Request::new("get-screen-size"),
        Command::GetMousePosition => Request::new("get-mouse-position"),
        Command::Screenshot { path, annotate } => {
            let mut req = Request::new("screenshot")
                .with_extra("annotate", json!(annotate));
            if let Some(p) = path {
                req = req.with_extra("path", json!(p.to_string_lossy()));
            }
            req
        }
        Command::Launch { command, args } => {
            Request::new("launch")
                .with_extra("command", json!(command))
                .with_extra("args", json!(args))
        }
        Command::Daemon(_) => unreachable!(),
    };
    Ok(req)
}

fn print_response(cmd: &Command, response: &Response) -> Result<()> {
    if !response.success {
        if let Some(ref err) = response.error {
            eprintln!("Error: {err}");
        }
        std::process::exit(1);
    }
    if let Some(ref data) = response.data {
        // For snapshot, print compact text format
        if matches!(cmd, Command::Snapshot { .. }) {
            if let Some(screenshot) = data.get("screenshot").and_then(|v| v.as_str()) {
                println!("Screenshot: {screenshot}");
            }
            if let Some(windows) = data.get("windows").and_then(|v| v.as_array()) {
                println!("Windows:");
                for w in windows {
                    let ref_id = w.get("ref_id").and_then(|v| v.as_str()).unwrap_or("?");
                    let title = w.get("title").and_then(|v| v.as_str()).unwrap_or("");
                    let focused = w.get("focused").and_then(|v| v.as_bool()).unwrap_or(false);
                    let minimized = w.get("minimized").and_then(|v| v.as_bool()).unwrap_or(false);
                    let x = w.get("x").and_then(|v| v.as_i64()).unwrap_or(0);
                    let y = w.get("y").and_then(|v| v.as_i64()).unwrap_or(0);
                    let width = w.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
                    let height = w.get("height").and_then(|v| v.as_u64()).unwrap_or(0);
                    let state = if focused { "focused" } else if minimized { "hidden" } else { "visible" };
                    let display_title = if title.len() > 30 {
                        format!("{}...", &title[..27])
                    } else {
                        title.to_string()
                    };
                    println!("@{:<4} {:<30} ({:<7})  {},{} {}x{}", ref_id, display_title, state, x, y, width, height);
                }
            }
        } else {
            // Generic: print JSON data
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }
    Ok(())
}
