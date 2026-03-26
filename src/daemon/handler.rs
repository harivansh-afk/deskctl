use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};

use super::state::DaemonState;
use crate::backend::annotate::annotate_screenshot;
use crate::core::protocol::{Request, Response};
use crate::core::refs::{ResolveResult, SelectorQuery};
use crate::core::types::{MonitorInfo, ScreenSize, Snapshot, SystemInfo, VersionInfo, WindowInfo};

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
        "get-active-window" => handle_get_active_window(state).await,
        "get-monitors" => handle_get_monitors(state).await,
        "get-version" => handle_get_version(state).await,
        "get-systeminfo" => handle_get_systeminfo(state).await,
        "wait-window" => handle_wait(request, state, WaitKind::Window).await,
        "wait-focus" => handle_wait(request, state, WaitKind::Focus).await,
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
    let selector_query = SelectorQuery::parse(&selector);

    if let Some((x, y)) = parse_coords(&selector) {
        return match state.backend.click(x, y) {
            Ok(()) => Response::ok(serde_json::json!({"clicked": {"x": x, "y": y}})),
            Err(error) => Response::err(format!("Click failed: {error}")),
        };
    }

    if selector_query.needs_live_refresh() {
        if let Err(error) = refresh_windows(&mut state) {
            return Response::err(format!("Click failed: {error}"));
        }
    }

    match state.ref_map.resolve_to_center(&selector) {
        ResolveResult::Match(entry) => {
            let (x, y) = entry.center();
            match state.backend.click(x, y) {
                Ok(()) => Response::ok(serde_json::json!({
                    "clicked": {"x": x, "y": y},
                    "selector": selector,
                    "ref_id": entry.ref_id,
                    "window_id": entry.window_id,
                    "title": entry.title,
                })),
                Err(error) => Response::err(format!("Click failed: {error}")),
            }
        }
        outcome => selector_failure_response(outcome),
    }
}

async fn handle_dblclick(request: &Request, state: &Arc<Mutex<DaemonState>>) -> Response {
    let selector = match request.extra.get("selector").and_then(|v| v.as_str()) {
        Some(selector) => selector.to_string(),
        None => return Response::err("Missing 'selector' field"),
    };

    let mut state = state.lock().await;
    let selector_query = SelectorQuery::parse(&selector);

    if let Some((x, y)) = parse_coords(&selector) {
        return match state.backend.dblclick(x, y) {
            Ok(()) => Response::ok(serde_json::json!({"double_clicked": {"x": x, "y": y}})),
            Err(error) => Response::err(format!("Double-click failed: {error}")),
        };
    }

    if selector_query.needs_live_refresh() {
        if let Err(error) = refresh_windows(&mut state) {
            return Response::err(format!("Double-click failed: {error}"));
        }
    }

    match state.ref_map.resolve_to_center(&selector) {
        ResolveResult::Match(entry) => {
            let (x, y) = entry.center();
            match state.backend.dblclick(x, y) {
                Ok(()) => Response::ok(serde_json::json!({
                    "double_clicked": {"x": x, "y": y},
                    "selector": selector,
                    "ref_id": entry.ref_id,
                    "window_id": entry.window_id,
                    "title": entry.title,
                })),
                Err(error) => Response::err(format!("Double-click failed: {error}")),
            }
        }
        outcome => selector_failure_response(outcome),
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
    let selector_query = SelectorQuery::parse(&selector);
    if selector_query.needs_live_refresh() {
        if let Err(error) = refresh_windows(&mut state) {
            return Response::err(format!("{action} failed: {error}"));
        }
    }
    let entry = match state.ref_map.resolve(&selector) {
        ResolveResult::Match(entry) => entry,
        outcome => return selector_failure_response(outcome),
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
            "title": entry.title,
            "ref_id": entry.ref_id,
            "window_id": entry.window_id,
            "selector": selector,
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
    let selector_query = SelectorQuery::parse(&selector);
    if selector_query.needs_live_refresh() {
        if let Err(error) = refresh_windows(&mut state) {
            return Response::err(format!("Move failed: {error}"));
        }
    }
    let entry = match state.ref_map.resolve(&selector) {
        ResolveResult::Match(entry) => entry,
        outcome => return selector_failure_response(outcome),
    };

    match state.backend.move_window(entry.backend_window_id, x, y) {
        Ok(()) => Response::ok(serde_json::json!({
            "moved": entry.title,
            "title": entry.title,
            "ref_id": entry.ref_id,
            "window_id": entry.window_id,
            "selector": selector,
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
    let selector_query = SelectorQuery::parse(&selector);
    if selector_query.needs_live_refresh() {
        if let Err(error) = refresh_windows(&mut state) {
            return Response::err(format!("Resize failed: {error}"));
        }
    }
    let entry = match state.ref_map.resolve(&selector) {
        ResolveResult::Match(entry) => entry,
        outcome => return selector_failure_response(outcome),
    };

    match state
        .backend
        .resize_window(entry.backend_window_id, width, height)
    {
        Ok(()) => Response::ok(serde_json::json!({
            "resized": entry.title,
            "title": entry.title,
            "ref_id": entry.ref_id,
            "window_id": entry.window_id,
            "selector": selector,
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

async fn handle_get_active_window(state: &Arc<Mutex<DaemonState>>) -> Response {
    let mut state = state.lock().await;
    let active_backend_window = match state.backend.active_window() {
        Ok(window) => window,
        Err(error) => return Response::err(format!("Failed: {error}")),
    };

    let windows = match refresh_windows(&mut state) {
        Ok(windows) => windows,
        Err(error) => return Response::err(format!("Failed: {error}")),
    };

    let active_window = if let Some(active_backend_window) = active_backend_window {
        state
            .ref_map
            .entries()
            .find_map(|(_, entry)| {
                (entry.backend_window_id == active_backend_window.native_id)
                    .then(|| entry.to_window_info())
            })
            .or_else(|| windows.iter().find(|window| window.focused).cloned())
    } else {
        windows.iter().find(|window| window.focused).cloned()
    };

    if let Some(window) = active_window {
        Response::ok(serde_json::json!({"window": window}))
    } else {
        Response::err_with_data(
            "No focused window is available",
            serde_json::json!({"kind": "not_found", "mode": "focused"}),
        )
    }
}

async fn handle_get_monitors(state: &Arc<Mutex<DaemonState>>) -> Response {
    let state = state.lock().await;
    match state.backend.list_monitors() {
        Ok(monitors) => {
            let monitors: Vec<MonitorInfo> = monitors.into_iter().map(Into::into).collect();
            Response::ok(serde_json::json!({
                "count": monitors.len(),
                "monitors": monitors,
            }))
        }
        Err(error) => Response::err(format!("Failed: {error}")),
    }
}

async fn handle_get_version(state: &Arc<Mutex<DaemonState>>) -> Response {
    let state = state.lock().await;
    let info = VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        backend: state.backend.backend_name().to_string(),
    };
    Response::ok(serde_json::to_value(info).unwrap_or_default())
}

async fn handle_get_systeminfo(state: &Arc<Mutex<DaemonState>>) -> Response {
    let state = state.lock().await;
    let screen = match state.backend.screen_size() {
        Ok((width, height)) => ScreenSize { width, height },
        Err(error) => return Response::err(format!("Failed: {error}")),
    };
    let monitors = match state.backend.list_monitors() {
        Ok(monitors) => monitors.into_iter().map(Into::into).collect::<Vec<_>>(),
        Err(error) => return Response::err(format!("Failed: {error}")),
    };

    let info = SystemInfo {
        backend: state.backend.backend_name().to_string(),
        display: std::env::var("DISPLAY")
            .ok()
            .filter(|value| !value.is_empty()),
        session_type: std::env::var("XDG_SESSION_TYPE")
            .ok()
            .filter(|value| !value.is_empty()),
        session: state.session.clone(),
        socket_path: state.socket_path.display().to_string(),
        screen,
        monitor_count: monitors.len(),
        monitors,
    };

    Response::ok(serde_json::to_value(info).unwrap_or_default())
}

async fn handle_wait(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
    wait_kind: WaitKind,
) -> Response {
    let selector = match request.extra.get("selector").and_then(|v| v.as_str()) {
        Some(selector) => selector.to_string(),
        None => return Response::err("Missing 'selector' field"),
    };
    let timeout_ms = request
        .extra
        .get("timeout_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(10_000);
    let poll_ms = request
        .extra
        .get("poll_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(250);

    let start = Instant::now();
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    let mut last_observation: serde_json::Value;

    loop {
        let outcome = {
            let mut state = state.lock().await;
            if let Err(error) = refresh_windows(&mut state) {
                return Response::err(format!("Wait failed: {error}"));
            }
            observe_wait(&state, &selector, wait_kind)
        };

        match outcome {
            WaitObservation::Satisfied(window) => {
                let elapsed_ms = start.elapsed().as_millis() as u64;
                return Response::ok(serde_json::json!({
                    "wait": wait_kind.as_str(),
                    "selector": selector,
                    "elapsed_ms": elapsed_ms,
                    "window": window,
                }));
            }
            WaitObservation::Retry { observation } => {
                last_observation = observation;
            }
            WaitObservation::Failure(response) => return response,
        }

        if Instant::now() >= deadline {
            return Response::err_with_data(
                format!(
                    "Timed out waiting for {} to match selector: {}",
                    wait_kind.as_str(),
                    selector
                ),
                serde_json::json!({
                    "kind": "timeout",
                    "wait": wait_kind.as_str(),
                    "selector": selector,
                    "timeout_ms": timeout_ms,
                    "poll_ms": poll_ms,
                    "last_observation": last_observation,
                }),
            );
        }

        sleep(Duration::from_millis(poll_ms)).await;
    }
}

#[derive(Clone, Copy)]
enum WaitKind {
    Window,
    Focus,
}

impl WaitKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Window => "window",
            Self::Focus => "focus",
        }
    }
}

enum WaitObservation {
    Satisfied(WindowInfo),
    Retry { observation: serde_json::Value },
    Failure(Response),
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

fn selector_failure_response(result: ResolveResult) -> Response {
    match result {
        ResolveResult::NotFound { selector, mode } => Response::err_with_data(
            format!("Could not resolve selector: {selector}"),
            serde_json::json!({
                "kind": "selector_not_found",
                "selector": selector,
                "mode": mode,
            }),
        ),
        ResolveResult::Ambiguous {
            selector,
            mode,
            candidates,
        } => Response::err_with_data(
            format!("Selector is ambiguous: {selector}"),
            serde_json::json!({
                "kind": "selector_ambiguous",
                "selector": selector,
                "mode": mode,
                "candidates": candidates,
            }),
        ),
        ResolveResult::Invalid {
            selector,
            mode,
            message,
        } => Response::err_with_data(
            format!("Invalid selector '{selector}': {message}"),
            serde_json::json!({
                "kind": "selector_invalid",
                "selector": selector,
                "mode": mode,
                "message": message,
            }),
        ),
        ResolveResult::Match(_) => unreachable!(),
    }
}

fn observe_wait(state: &DaemonState, selector: &str, wait_kind: WaitKind) -> WaitObservation {
    match state.ref_map.resolve(selector) {
        ResolveResult::Match(entry) => {
            let window = entry.to_window_info();
            match wait_kind {
                WaitKind::Window => WaitObservation::Satisfied(window),
                WaitKind::Focus if window.focused => WaitObservation::Satisfied(window),
                WaitKind::Focus => WaitObservation::Retry {
                    observation: serde_json::json!({
                        "kind": "window_not_focused",
                        "window": window,
                    }),
                },
            }
        }
        ResolveResult::NotFound { selector, mode } => WaitObservation::Retry {
            observation: serde_json::json!({
                "kind": "selector_not_found",
                "selector": selector,
                "mode": mode,
            }),
        },
        ResolveResult::Ambiguous {
            selector,
            mode,
            candidates,
        } => WaitObservation::Failure(Response::err_with_data(
            format!("Selector is ambiguous: {selector}"),
            serde_json::json!({
                "kind": "selector_ambiguous",
                "selector": selector,
                "mode": mode,
                "candidates": candidates,
            }),
        )),
        ResolveResult::Invalid {
            selector,
            mode,
            message,
        } => WaitObservation::Failure(Response::err_with_data(
            format!("Invalid selector '{selector}': {message}"),
            serde_json::json!({
                "kind": "selector_invalid",
                "selector": selector,
                "mode": mode,
                "message": message,
            }),
        )),
    }
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

impl From<crate::backend::BackendMonitor> for MonitorInfo {
    fn from(value: crate::backend::BackendMonitor) -> Self {
        Self {
            name: value.name,
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
            width_mm: value.width_mm,
            height_mm: value.height_mm,
            primary: value.primary,
            automatic: value.automatic,
        }
    }
}
