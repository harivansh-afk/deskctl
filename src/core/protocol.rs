use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub action: String,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Request {
    pub fn new(action: &str) -> Self {
        Self {
            id: format!(
                "r{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_micros()
                    % 1_000_000
            ),
            action: action.to_string(),
            extra: Value::Object(serde_json::Map::new()),
        }
    }

    pub fn with_extra(mut self, key: &str, value: Value) -> Self {
        if let Value::Object(ref mut map) = self.extra {
            map.insert(key.to_string(), value);
        }
        self
    }
}

impl Response {
    pub fn ok(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }

    pub fn err_with_data(msg: impl Into<String>, data: Value) -> Self {
        Self {
            success: false,
            data: Some(data),
            error: Some(msg.into()),
        }
    }
}
