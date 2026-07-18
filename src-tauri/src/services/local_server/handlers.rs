use super::types::MessageRequest;
use super::types::OpenTrayRequest;
use super::types::PingResponse;
use crate::models::system::SyncTokenRequest;
use crate::models::system::TokenState;
use crate::state::APP_HANDLE;
use crate::state::APP_STATE;
use crate::state::CURRENT_ROUTE;
use crate::utils::notification;
use axum::response::IntoResponse;
use axum::Json;
use tauri::{Emitter, Manager};

pub async fn exit_app() -> &'static str {
    if let Some(app) = APP_HANDLE.get() {
        let app = app.clone();

        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Dừng các tác vụ nếu cần
            // crate::state::update_sync_emit(|s| {
            //     s.source.clear();
            //     s.running = false;
            //     s.current_invoice = None;
            // });

            app.exit(0);
        });
    }

    "OK"
}

pub async fn ping() -> Json<PingResponse> {
    // if let Err(err) = notification::show("vaOne", "Kết nối thành công!") {
    //     eprintln!("Show notification failed: {}", err);
    // }
    Json(PingResponse { success: true })
}

pub async fn message(Json(req): Json<MessageRequest>) -> Json<PingResponse> {
    if let Err(err) = notification::show("vaOne", &req.message) {
        eprintln!("Show notification failed: {}", err);
    }
    Json(PingResponse { success: true })
}

pub async fn sync_token(Json(req): Json<SyncTokenRequest>) -> impl IntoResponse {
    println!(
        "Tenant {} has token = {}",
        req.tenant_id,
        req.token.is_some()
    );
    update_token(req);
    Json(serde_json::json!({
        "success": true
    }))
}

pub async fn open_tray_page(Json(req): Json<OpenTrayRequest>) -> Json<serde_json::Value> {
    let payload = serde_json::json!({
        "route": req.route,
        "data": req.data
    });
    if let Some(app) = APP_HANDLE.get() {
        if let Some(window) = app.get_webview_window("main") {
            let current = CURRENT_ROUTE.get().unwrap().lock().unwrap().clone();
            let route = payload["route"].as_str().unwrap_or("");
            if current == route {
                let _ = window.set_focus();
            } else {
                let _ = window.emit("tray-navigate", payload);
            }
        }
    }
    Json(serde_json::json!({
        "success": true
    }))
}

pub fn update_token(req: SyncTokenRequest) {
    let mut state = APP_STATE
        .get()
        .expect("APP_STATE not initialized")
        .lock()
        .unwrap();

    let tenant = state.tenants.entry(req.tenant_id.clone()).or_default();

    // Luôn cập nhật auth config
    tenant.auth = Some(req.auth);
    match req.token {
        Some(token) => {
            let should_update = match &tenant.token {
                Some(current) => token.version > current.version,
                None => true,
            };

            if should_update {
                tenant.token = Some(token);
            }
        }

        None => {
            // Web logout
            tenant.token = None;
        }
    }
}
