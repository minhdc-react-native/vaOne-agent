use super::types::MessageRequest;
use super::types::OpenTrayRequest;
use super::types::PingResponse;
use crate::state::APP_HANDLE;
use crate::utils::notification;
use axum::Json;

use tauri::{Emitter, LogicalSize, Manager, Size};

pub async fn ping() -> Json<PingResponse> {
    if let Err(err) = notification::show("vaOne", "Kết nối thành công!") {
        eprintln!("Show notification failed: {}", err);
    }
    Json(PingResponse { success: true })
}

pub async fn message(Json(req): Json<MessageRequest>) -> Json<PingResponse> {
    if let Err(err) = notification::show("vaOne", &req.message) {
        eprintln!("Show notification failed: {}", err);
    }
    Json(PingResponse { success: true })
}

pub async fn open_tray_page(Json(req): Json<OpenTrayRequest>) -> Json<serde_json::Value> {
    let payload = serde_json::json!({
        "route": req.route,
        "data": req.data
    });
    if let Some(app) = APP_HANDLE.get() {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.emit("tray-navigate", payload);
        }
    }
    Json(serde_json::json!({
        "success": true
    }))
}
