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

    // eprintln!("payload>>: {}", payload);

    if let Some(app) = APP_HANDLE.get() {
        let width = req.data["screen"]["width"].as_f64().unwrap_or(380.0);
        let height = req.data["screen"]["height"].as_f64().unwrap_or(520.0);
        let title = req.data["screen"]["title"]
            .as_str()
            .unwrap_or("VAOne plugin");
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_title(title);
            let _ = window.set_size(Size::Logical(LogicalSize { width, height }));
            let _ = window.emit("tray-navigate", payload);
            if req.route != "/blank" {
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    let _ = window.center();
                    let _ = window.show();
                    let _ = window.set_focus();
                });
            }
        }
    }

    Json(serde_json::json!({
        "success": true
    }))
}
