use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::Mutex;

use super::state::DaemonState;
use crate::backend::annotate::annotate_screenshot;
use crate::core::protocol::{Request, Response};
use crate::core::types::{Snapshot, WindowInfo};

pub async fn handle_request(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    match request.action.as_str() {
        "ping" => Response::ok(serde_json::json!({"message": "pong"})),
        "snapshot" => handle_snapshot(request, state).await,
        "click" => handle_click(request, state).await,
        "dblclick" => handle_dblclick(request, state).await,
        "type" => handle_type(request, state).await,
        "press" => handle_press(request, state).await,
        "hotkey" => handle_hotkey(request, state).await,
        "mouse-move" => handle_mouse_move(request, state).await,
        "mouse-scroll" => handle_mouse_scroll(request, state).await,
        "mouse-drag" => handle_mouse_drag(request, state).await,
        "focus" => handle_window_action(request, state, "focus").await,
        "close" => handle_window_action(request, state, "close").await,
        "move-window" => handle_move_window(request, state).await,
        "resize-window" => handle_resize_window(request, state).await,
        "list-windows" => handle_list_windows(state).await,
        "get-screen-size" => handle_get_screen_size(state).await,
        "get-mouse-position" => handle_get_mouse_position(state).await,
        "screenshot" => handle_screenshot(request, state).await,
        "launch" => handle_launch(request, state).await,
        action => Response::err(format!("Unknown action: {action}")),
    }
}

async fn handle_snapshot(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let annotate = request
        .extra
        .get("annotate")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut state = state.lock().await;
    match capture_snapshot(&mut state, annotate, None) {
        Ok(snapshot) => Response::ok(serde_json::to_value(&snapshot).unwrap_or_default()),
        Err(error) => Response::err(format!("Snapshot failed: {error}")),
    }
}

async fn handle_click(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let selector = match request.extra.get("selector").and_then(|v| v.as_str()) {
        Some(selector) => selector.to_string(),
        None => return Response::err("Missing 'selector' field"),
    };

    let mut state = state.lock().await;

    if let Some((x, y)) = parse_coords(&selector) {
        return match state.backend.click(x, y) {
            Ok(()) => Response::ok(serde_json::json!({"clicked": {"x": x, "y": y}})),
            Err(error) => Response::err(format!("Click failed: {error}")),
        };
    }

    match state.ref_map.resolve_to_center(&selector) {
        Some((x, y)) => match state.backend.click(x, y) {
            Ok(()) => {
                Response::ok(serde_json::json!({"clicked": {"x": x, "y": y, "ref": selector}}))
            }
            Err(error) => Response::err(format!("Click failed: {error}")),
        },
        None => Response::err(format!("Could not resolve selector: {selector}")),
    }
}

async fn handle_dblclick(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let selector = match request.extra.get("selector").and_then(|v| v.as_str()) {
        Some(selector) => selector.to_string(),
        None => return Response::err("Missing 'selector' field"),
    };

    let mut state = state.lock().await;

    if let Some((x, y)) = parse_coords(&selector) {
        return match state.backend.dblclick(x, y) {
            Ok(()) => Response::ok(serde_json::json!({"double_clicked": {"x": x, "y": y}})),
            Err(error) => Response::err(format!("Double-click failed: {error}")),
        };
    }

    match state.ref_map.resolve_to_center(&selector) {
        Some((x, y)) => match state.backend.dblclick(x, y) {
            Ok(()) => Response::ok(
                serde_json::json!({"double_clicked": {"x": x, "y": y, "ref": selector}}),
            ),
            Err(error) => Response::err(format!("Double-click failed: {error}")),
        },
        None => Response::err(format!("Could not resolve selector: {selector}")),
    }
}

async fn handle_type(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let text = match request.extra.get("text").and_then(|v| v.as_str()) {
        Some(text) => text.to_string(),
        None => return Response::err("Missing 'text' field"),
    };

    let mut state = state.lock().await;
    match state.backend.type_text(&text) {
        Ok(()) => Response::ok(serde_json::json!({"typed": text})),
        Err(error) => Response::err(format!("Type failed: {error}")),
    }
}

async fn handle_press(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let key = match request.extra.get("key").and_then(|v| v.as_str()) {
        Some(key) => key.to_string(),
        None => return Response::err("Missing 'key' field"),
    };

    let mut state = state.lock().await;
    match state.backend.press_key(&key) {
        Ok(()) => Response::ok(serde_json::json!({"pressed": key})),
        Err(error) => Response::err(format!("Key press failed: {error}")),
    }
}

async fn handle_hotkey(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let keys: Vec<String> = match request.extra.get("keys").and_then(|v| v.as_array()) {
        Some(keys) => keys
            .iter()
            .filter_map(|value| value.as_str().map(|s| s.to_string()))
            .collect(),
        None => return Response::err("Missing 'keys' field"),
    };

    let mut state = state.lock().await;
    match state.backend.hotkey(&keys) {
        Ok(()) => Response::ok(serde_json::json!({"hotkey": keys})),
        Err(error) => Response::err(format!("Hotkey failed: {error}")),
    }
}

async fn handle_mouse_move(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let x = match request.extra.get("x").and_then(|v| v.as_i64()) {
        Some(value) => value as i32,
        None => return Response::err("Missing 'x' field"),
    };
    let y = match request.extra.get("y").and_then(|v| v.as_i64()) {
        Some(value) => value as i32,
        None => return Response::err("Missing 'y' field"),
    };

    let mut state = state.lock().await;
    match state.backend.mouse_move(x, y) {
        Ok(()) => Response::ok(serde_json::json!({"moved": {"x": x, "y": y}})),
        Err(error) => Response::err(format!("Mouse move failed: {error}")),
    }
}

async fn handle_mouse_scroll(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let amount = match request.extra.get("amount").and_then(|v| v.as_i64()) {
        Some(value) => value as i32,
        None => return Response::err("Missing 'amount' field"),
    };
    let axis = request
        .extra
        .get("axis")
        .and_then(|v| v.as_str())
        .unwrap_or("vertical")
        .to_string();

    let mut state = state.lock().await;
    match state.backend.scroll(amount, &axis) {
        Ok(()) => Response::ok(serde_json::json!({"scrolled": {"amount": amount, "axis": axis}})),
        Err(error) => Response::err(format!("Scroll failed: {error}")),
    }
}

async fn handle_mouse_drag(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let x1 = match request.extra.get("x1").and_then(|v| v.as_i64()) {
        Some(value) => value as i32,
        None => return Response::err("Missing 'x1' field"),
    };
    let y1 = match request.extra.get("y1").and_then(|v| v.as_i64()) {
        Some(value) => value as i32,
        None => return Response::err("Missing 'y1' field"),
    };
    let x2 = match request.extra.get("x2").and_then(|v| v.as_i64()) {
        Some(value) => value as i32,
        None => return Response::err("Missing 'x2' field"),
    };
    let y2 = match request.extra.get("y2").and_then(|v| v.as_i64()) {
        Some(value) => value as i32,
        None => return Response::err("Missing 'y2' field"),
    };

    let mut state = state.lock().await;
    match state.backend.drag(x1, y1, x2, y2) {
        Ok(()) => Response::ok(serde_json::json!({
            "dragged": {
                "from": {"x": x1, "y": y1},
                "to": {"x": x2, "y": y2}
            }
        })),
        Err(error) => Response::err(format!("Drag failed: {error}")),
    }
}

async fn handle_window_action(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
    action: &str,
) -> Response {
    let selector = match request.extra.get("selector").and_then(|v| v.as_str()) {
        Some(selector) => selector.to_string(),
        None => return Response::err("Missing 'selector' field"),
    };

    let mut state = state.lock().await;
    let entry = match state.ref_map.resolve(&selector) {
        Some(entry) => entry.clone(),
        None => return Response::err(format!("Could not resolve window: {selector}")),
    };

    let result = match action {
        "focus" => state.backend.focus_window(entry.backend_window_id),
        "close" => state.backend.close_window(entry.backend_window_id),
        _ => unreachable!(),
    };

    match result {
        Ok(()) => Response::ok(serde_json::json!({
            "action": action,
            "window": entry.title,
            "window_id": entry.window_id,
        })),
        Err(error) => Response::err(format!("{action} failed: {error}")),
    }
}

async fn handle_move_window(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let selector = match request.extra.get("selector").and_then(|v| v.as_str()) {
        Some(selector) => selector.to_string(),
        None => return Response::err("Missing 'selector' field"),
    };
    let x = request.extra.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let y = request.extra.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let mut state = state.lock().await;
    let entry = match state.ref_map.resolve(&selector) {
        Some(entry) => entry.clone(),
        None => return Response::err(format!("Could not resolve window: {selector}")),
    };

    match state.backend.move_window(entry.backend_window_id, x, y) {
        Ok(()) => Response::ok(serde_json::json!({
            "moved": entry.title,
            "window_id": entry.window_id,
            "x": x,
            "y": y,
        })),
        Err(error) => Response::err(format!("Move failed: {error}")),
    }
}

async fn handle_resize_window(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let selector = match request.extra.get("selector").and_then(|v| v.as_str()) {
        Some(selector) => selector.to_string(),
        None => return Response::err("Missing 'selector' field"),
    };
    let width = request
        .extra
        .get("w")
        .and_then(|v| v.as_u64())
        .unwrap_or(800) as u32;
    let height = request
        .extra
        .get("h")
        .and_then(|v| v.as_u64())
        .unwrap_or(600) as u32;

    let mut state = state.lock().await;
    let entry = match state.ref_map.resolve(&selector) {
        Some(entry) => entry.clone(),
        None => return Response::err(format!("Could not resolve window: {selector}")),
    };

    match state
        .backend
        .resize_window(entry.backend_window_id, width, height)
    {
        Ok(()) => Response::ok(serde_json::json!({
            "resized": entry.title,
            "window_id": entry.window_id,
            "width": width,
            "height": height,
        })),
        Err(error) => Response::err(format!("Resize failed: {error}")),
    }
}

async fn handle_list_windows(state: &Arc<Mutex<DaemonState>>) -> Response {
    let mut state = state.lock().await;
    match refresh_windows(&mut state) {
        Ok(windows) => Response::ok(serde_json::json!({"windows": windows})),
        Err(error) => Response::err(format!("List windows failed: {error}")),
    }
}

async fn handle_get_screen_size(state: &Arc<Mutex<DaemonState>>) -> Response {
    let state = state.lock().await;
    match state.backend.screen_size() {
        Ok((width, height)) => Response::ok(serde_json::json!({"width": width, "height": height})),
        Err(error) => Response::err(format!("Failed: {error}")),
    }
}

async fn handle_get_mouse_position(state: &Arc<Mutex<DaemonState>>) -> Response {
    let state = state.lock().await;
    match state.backend.mouse_position() {
        Ok((x, y)) => Response::ok(serde_json::json!({"x": x, "y": y})),
        Err(error) => Response::err(format!("Failed: {error}")),
    }
}

async fn handle_screenshot(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let annotate = request
        .extra
        .get("annotate")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let path = request
        .extra
        .get("path")
        .and_then(|v| v.as_str())
        .map(|value| value.to_string())
        .unwrap_or_else(temp_screenshot_path);

    let mut state = state.lock().await;
    let windows = if annotate {
        match refresh_windows(&mut state) {
            Ok(windows) => Some(windows),
            Err(error) => return Response::err(format!("Screenshot failed: {error}")),
        }
    } else {
        None
    };

    match capture_and_save_screenshot(&mut state, &path, annotate, windows.as_deref()) {
        Ok(saved) => {
            if let Some(windows) = windows {
                Response::ok(serde_json::json!({"screenshot": saved, "windows": windows}))
            } else {
                Response::ok(serde_json::json!({"screenshot": saved}))
            }
        }
        Err(error) => Response::err(format!("Screenshot failed: {error}")),
    }
}

async fn handle_launch(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let command = match request.extra.get("command").and_then(|v| v.as_str()) {
        Some(command) => command.to_string(),
        None => return Response::err("Missing 'command' field"),
    };
    let args: Vec<String> = request
        .extra
        .get("args")
        .and_then(|v| v.as_array())
        .map(|args| {
            args.iter()
                .filter_map(|value| value.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let state = state.lock().await;
    match state.backend.launch(&command, &args) {
        Ok(pid) => Response::ok(serde_json::json!({"pid": pid, "command": command})),
        Err(error) => Response::err(format!("Launch failed: {error}")),
    }
}

fn refresh_windows(state: &mut DaemonState) -> Result<Vec<WindowInfo>> {
    let windows = state.backend.list_windows()?;
    Ok(state.ref_map.rebuild(&windows))
}

fn capture_snapshot(
    state: &mut DaemonState,
    annotate: bool,
    path: Option<String>,
) -> Result<Snapshot> {
    let windows = refresh_windows(state)?;
    let screenshot_path = path.unwrap_or_else(temp_screenshot_path);
    let screenshot =
        capture_and_save_screenshot(state, &screenshot_path, annotate, Some(&windows))?;

    Ok(Snapshot {
        screenshot,
        windows,
    })
}

fn capture_and_save_screenshot(
    state: &mut DaemonState,
    path: &str,
    annotate: bool,
    windows: Option<&[WindowInfo]>,
) -> Result<String> {
    let mut image = state.backend.capture_screenshot()?;
    if annotate {
        let windows = windows.context("Annotated screenshots require current window data")?;
        annotate_screenshot(&mut image, windows);
    }
    image
        .save(path)
        .with_context(|| format!("Failed to save screenshot to {path}"))?;
    Ok(path.to_string())
}

fn temp_screenshot_path() -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("/tmp/deskctl-{timestamp}.png")
}

fn parse_coords(value: &str) -> Option<(i32, i32)> {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() != 2 {
        return None;
    }

    let x = parts[0].trim().parse().ok()?;
    let y = parts[1].trim().parse().ok()?;
    Some((x, y))
}
