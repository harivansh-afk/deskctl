use std::sync::Arc;
use tokio::sync::Mutex;

use crate::backend::DesktopBackend;
use crate::core::protocol::{Request, Response};
use crate::core::refs::RefEntry;
use super::state::DaemonState;

pub async fn handle_request(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    match request.action.as_str() {
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

async fn handle_snapshot(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let annotate = request
        .extra
        .get("annotate")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let mut state = state.lock().await;

    match state.backend.snapshot(annotate) {
        Ok(snapshot) => {
            // Update ref map
            state.ref_map.clear();
            for win in &snapshot.windows {
                state.ref_map.insert(RefEntry {
                    xcb_id: win.xcb_id,
                    app_class: win.app_name.clone(),
                    title: win.title.clone(),
                    pid: 0, // xcap doesn't expose PID directly in snapshot
                    x: win.x,
                    y: win.y,
                    width: win.width,
                    height: win.height,
                    focused: win.focused,
                    minimized: win.minimized,
                });
            }

            Response::ok(serde_json::to_value(&snapshot).unwrap_or_default())
        }
        Err(e) => Response::err(format!("Snapshot failed: {e}")),
    }
}

async fn handle_click(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let selector = match request.extra.get("selector").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return Response::err("Missing 'selector' field"),
    };

    let mut state = state.lock().await;

    // Try to parse as coordinates "x,y"
    if let Some((x, y)) = parse_coords(&selector) {
        return match state.backend.click(x, y) {
            Ok(()) => Response::ok(serde_json::json!({"clicked": {"x": x, "y": y}})),
            Err(e) => Response::err(format!("Click failed: {e}")),
        };
    }

    // Resolve as window ref
    match state.ref_map.resolve_to_center(&selector) {
        Some((x, y)) => match state.backend.click(x, y) {
            Ok(()) => Response::ok(
                serde_json::json!({"clicked": {"x": x, "y": y, "ref": selector}}),
            ),
            Err(e) => Response::err(format!("Click failed: {e}")),
        },
        None => Response::err(format!("Could not resolve selector: {selector}")),
    }
}

async fn handle_dblclick(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let selector = match request.extra.get("selector").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return Response::err("Missing 'selector' field"),
    };

    let mut state = state.lock().await;

    if let Some((x, y)) = parse_coords(&selector) {
        return match state.backend.dblclick(x, y) {
            Ok(()) => Response::ok(serde_json::json!({"double_clicked": {"x": x, "y": y}})),
            Err(e) => Response::err(format!("Double-click failed: {e}")),
        };
    }

    match state.ref_map.resolve_to_center(&selector) {
        Some((x, y)) => match state.backend.dblclick(x, y) {
            Ok(()) => Response::ok(
                serde_json::json!({"double_clicked": {"x": x, "y": y, "ref": selector}}),
            ),
            Err(e) => Response::err(format!("Double-click failed: {e}")),
        },
        None => Response::err(format!("Could not resolve selector: {selector}")),
    }
}

async fn handle_type(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let text = match request.extra.get("text").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return Response::err("Missing 'text' field"),
    };

    let mut state = state.lock().await;

    match state.backend.type_text(&text) {
        Ok(()) => Response::ok(serde_json::json!({"typed": text})),
        Err(e) => Response::err(format!("Type failed: {e}")),
    }
}

async fn handle_press(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let key = match request.extra.get("key").and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => return Response::err("Missing 'key' field"),
    };

    let mut state = state.lock().await;

    match state.backend.press_key(&key) {
        Ok(()) => Response::ok(serde_json::json!({"pressed": key})),
        Err(e) => Response::err(format!("Key press failed: {e}")),
    }
}

async fn handle_hotkey(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let keys: Vec<String> = match request.extra.get("keys").and_then(|v| v.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        None => return Response::err("Missing 'keys' field"),
    };

    let mut state = state.lock().await;

    match state.backend.hotkey(&keys) {
        Ok(()) => Response::ok(serde_json::json!({"hotkey": keys})),
        Err(e) => Response::err(format!("Hotkey failed: {e}")),
    }
}

async fn handle_mouse_move(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let x = match request.extra.get("x").and_then(|v| v.as_i64()) {
        Some(v) => v as i32,
        None => return Response::err("Missing 'x' field"),
    };
    let y = match request.extra.get("y").and_then(|v| v.as_i64()) {
        Some(v) => v as i32,
        None => return Response::err("Missing 'y' field"),
    };

    let mut state = state.lock().await;

    match state.backend.mouse_move(x, y) {
        Ok(()) => Response::ok(serde_json::json!({"moved": {"x": x, "y": y}})),
        Err(e) => Response::err(format!("Mouse move failed: {e}")),
    }
}

async fn handle_mouse_scroll(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let amount = match request.extra.get("amount").and_then(|v| v.as_i64()) {
        Some(v) => v as i32,
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
        Ok(()) => {
            Response::ok(serde_json::json!({"scrolled": {"amount": amount, "axis": axis}}))
        }
        Err(e) => Response::err(format!("Scroll failed: {e}")),
    }
}

async fn handle_mouse_drag(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let x1 = match request.extra.get("x1").and_then(|v| v.as_i64()) {
        Some(v) => v as i32,
        None => return Response::err("Missing 'x1' field"),
    };
    let y1 = match request.extra.get("y1").and_then(|v| v.as_i64()) {
        Some(v) => v as i32,
        None => return Response::err("Missing 'y1' field"),
    };
    let x2 = match request.extra.get("x2").and_then(|v| v.as_i64()) {
        Some(v) => v as i32,
        None => return Response::err("Missing 'x2' field"),
    };
    let y2 = match request.extra.get("y2").and_then(|v| v.as_i64()) {
        Some(v) => v as i32,
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
        Err(e) => Response::err(format!("Drag failed: {e}")),
    }
}

async fn handle_window_action(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
    action: &str,
) -> Response {
    let selector = match request.extra.get("selector").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return Response::err("Missing 'selector' field"),
    };

    let mut state = state.lock().await;

    let entry = match state.ref_map.resolve(&selector) {
        Some(e) => e.clone(),
        None => return Response::err(format!("Could not resolve window: {selector}")),
    };

    let result = match action {
        "focus" => state.backend.focus_window(entry.xcb_id),
        "close" => state.backend.close_window(entry.xcb_id),
        _ => unreachable!(),
    };

    match result {
        Ok(()) => Response::ok(serde_json::json!({
            "action": action,
            "window": entry.title,
            "xcb_id": entry.xcb_id,
        })),
        Err(e) => Response::err(format!("{action} failed: {e}")),
    }
}

async fn handle_move_window(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let selector = match request.extra.get("selector").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return Response::err("Missing 'selector' field"),
    };
    let x = request.extra.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let y = request.extra.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    let mut state = state.lock().await;
    let entry = match state.ref_map.resolve(&selector) {
        Some(e) => e.clone(),
        None => return Response::err(format!("Could not resolve window: {selector}")),
    };

    match state.backend.move_window(entry.xcb_id, x, y) {
        Ok(()) => Response::ok(serde_json::json!({
            "moved": entry.title, "x": x, "y": y
        })),
        Err(e) => Response::err(format!("Move failed: {e}")),
    }
}

async fn handle_resize_window(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let selector = match request.extra.get("selector").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return Response::err("Missing 'selector' field"),
    };
    let w = request.extra.get("w").and_then(|v| v.as_u64()).unwrap_or(800) as u32;
    let h = request.extra.get("h").and_then(|v| v.as_u64()).unwrap_or(600) as u32;

    let mut state = state.lock().await;
    let entry = match state.ref_map.resolve(&selector) {
        Some(e) => e.clone(),
        None => return Response::err(format!("Could not resolve window: {selector}")),
    };

    match state.backend.resize_window(entry.xcb_id, w, h) {
        Ok(()) => Response::ok(serde_json::json!({
            "resized": entry.title, "width": w, "height": h
        })),
        Err(e) => Response::err(format!("Resize failed: {e}")),
    }
}

async fn handle_list_windows(
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let mut state = state.lock().await;
    // Re-run snapshot without screenshot, just to get current window list
    match state.backend.snapshot(false) {
        Ok(snapshot) => {
            // Update ref map with fresh data
            state.ref_map.clear();
            for win in &snapshot.windows {
                state.ref_map.insert(RefEntry {
                    xcb_id: win.xcb_id,
                    app_class: win.app_name.clone(),
                    title: win.title.clone(),
                    pid: 0,
                    x: win.x,
                    y: win.y,
                    width: win.width,
                    height: win.height,
                    focused: win.focused,
                    minimized: win.minimized,
                });
            }
            Response::ok(serde_json::json!({"windows": snapshot.windows}))
        }
        Err(e) => Response::err(format!("List windows failed: {e}")),
    }
}

async fn handle_get_screen_size(state: &Arc<Mutex<DaemonState>>) -> Response {
    let state = state.lock().await;
    match state.backend.screen_size() {
        Ok((w, h)) => Response::ok(serde_json::json!({"width": w, "height": h})),
        Err(e) => Response::err(format!("Failed: {e}")),
    }
}

async fn handle_get_mouse_position(state: &Arc<Mutex<DaemonState>>) -> Response {
    let state = state.lock().await;
    match state.backend.mouse_position() {
        Ok((x, y)) => Response::ok(serde_json::json!({"x": x, "y": y})),
        Err(e) => Response::err(format!("Failed: {e}")),
    }
}

async fn handle_screenshot(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let annotate = request
        .extra
        .get("annotate")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let path = request
        .extra
        .get("path")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            format!("/tmp/desktop-ctl-{ts}.png")
        });
    let mut state = state.lock().await;
    match state.backend.screenshot(&path, annotate) {
        Ok(saved) => Response::ok(serde_json::json!({"screenshot": saved})),
        Err(e) => Response::err(format!("Screenshot failed: {e}")),
    }
}

async fn handle_launch(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let command = match request.extra.get("command").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err("Missing 'command' field"),
    };
    let args: Vec<String> = request
        .extra
        .get("args")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let state = state.lock().await;
    match state.backend.launch(&command, &args) {
        Ok(pid) => Response::ok(serde_json::json!({"pid": pid, "command": command})),
        Err(e) => Response::err(format!("Launch failed: {e}")),
    }
}

fn parse_coords(s: &str) -> Option<(i32, i32)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() == 2 {
        let x = parts[0].trim().parse().ok()?;
        let y = parts[1].trim().parse().ok()?;
        Some((x, y))
    } else {
        None
    }
}
