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
        action => Response::err(format!("Unknown action: {action}")),
    }
}

async fn handle_snapshot(
    request: &Request,
    state: &Arc<Mutex<DaemonState>>,
) -> Response {
    let annotate = request.extra.get("annotate")
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
