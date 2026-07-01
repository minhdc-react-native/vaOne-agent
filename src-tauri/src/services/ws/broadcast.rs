use serde::Serialize;

use crate::state::WS_STATE;

/// Broadcast raw string tới tất cả client
pub fn broadcast(message: impl Into<String>) {
    if let Some(ws) = WS_STATE.get() {
        ws.broadcast(message.into());
    }
}

/// Broadcast object JSON
pub fn broadcast_json<T>(event: &str, data: &T)
where
    T: Serialize,
{
    let payload = serde_json::json!({
        "event": event,
        "data": data
    });

    if let Some(ws) = WS_STATE.get() {
        ws.broadcast(payload.to_string());
    }
}
