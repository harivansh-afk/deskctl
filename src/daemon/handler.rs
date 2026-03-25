use std::sync::Arc;
use tokio::sync::Mutex;

use crate::core::protocol::{Request, Response};
use super::state::DaemonState;

pub async fn handle_request(
    request: &Request,
    _state: &Arc<Mutex<DaemonState>>,
) -> Response {
    match request.action.as_str() {
        "snapshot" => {
            Response::ok(serde_json::json!({
                "screenshot": "/tmp/desktop-ctl-placeholder.png",
                "windows": [
                    {
                        "ref_id": "w1",
                        "xcb_id": 0,
                        "title": "Placeholder Window",
                        "app_name": "placeholder",
                        "x": 0, "y": 0, "width": 1920, "height": 1080,
                        "focused": true, "minimized": false
                    }
                ]
            }))
        }
        action => {
            Response::err(format!("Unknown action: {action}"))
        }
    }
}
