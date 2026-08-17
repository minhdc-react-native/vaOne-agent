use super::types::MessageRequest;
use super::types::OpenTrayRequest;
use super::types::PingResponse;
use crate::api::http::get_image_base64;
use crate::auth::token_manager::TokenManager;
use crate::models::system::PrintResponse;
use crate::models::system::SyncTokenRequest;
use crate::state::APP_HANDLE;
use crate::state::APP_STATE;
use crate::state::CURRENT_ROUTE;
use crate::utils::notification;
use crate::utils::public::decompress_zstd_json;
use axum::body::Body;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::Json;
use pdf_core::models::Element;
use pdf_core::models::PdfTemplate;
use pdf_core::renderer;
use printer_core::PrintOptions;
use reqwest::header::CONTENT_DISPOSITION;
use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;
use serde::Deserialize;
use std::env;
use std::fs;
use tauri::{Emitter, Manager};
use uuid::Uuid;

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
    TokenManager::sync(&req.tenant_id, req.token, Some(req.auth));

    let (source, username, tax_code) = APP_STATE
        .get()
        .and_then(|state| state.lock().ok())
        .and_then(|state| {
            state
                .tenants
                .get(&req.tenant_id)
                .and_then(|tenant| tenant.info_login.as_ref())
                .map(|login| {
                    (
                        login.source.clone(),
                        login.username.clone(),
                        login.tax_code.clone(),
                    )
                })
        })
        .unwrap_or_default();

    Json(serde_json::json!({
        "success": true,
        "source": source,
        "username": username,
        "taxCode": tax_code
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
#[derive(Deserialize)]
pub struct RenderPdfRequest {
    pub reports: Vec<u8>,
    pub datas: Vec<u8>,
    pub options: Option<PrintOptions>,
}

pub async fn render_pdf(
    Json(mut req): Json<RenderPdfRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Tạo file tạm
    let output = env::temp_dir().join(format!("{}.pdf", Uuid::new_v4()));

    let output_clone = output.clone();

    let mut reports: Vec<PdfTemplate> = decompress_zstd_json(&req.reports).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to decompress reports: {}", e),
        )
    })?;

    let datas: Vec<serde_json::Value> = decompress_zstd_json(&req.datas).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to decompress datas: {}", e),
        )
    })?;

    // Nếu muốn gửi progress qua websocket thì giữ callback này,
    // còn không thì để rỗng.
    let mut progress = |_p: serde_json::Value| {};
    prepare_reports(&mut reports)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tokio::task::spawn_blocking(move || {
        renderer::render_page(
            reports,
            datas,
            output_clone.to_str().unwrap(),
            &mut progress,
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if let Some(options) = req.options {
        match printer_core::print_pdf(options, output.to_str().unwrap()) {
            Ok(_) => {
                let _ = fs::remove_file(&output);

                let mut headers = HeaderMap::new();
                headers.insert("content-type", "application/json".parse().unwrap());

                let body = serde_json::to_vec(&PrintResponse {
                    success: true,
                    printed: true,
                    message: None,
                })
                .unwrap();

                return Ok((headers, Body::from(body)));
            }
            Err(e) => {
                let _ = fs::remove_file(&output);

                let mut headers = HeaderMap::new();
                headers.insert("content-type", "application/json".parse().unwrap());

                let body = serde_json::to_vec(&PrintResponse {
                    success: false,
                    printed: false,
                    message: Some(e.to_string()),
                })
                .unwrap();

                return Ok((headers, Body::from(body)));
            }
        }
    }

    // Đọc file PDF
    let bytes =
        fs::read(&output).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Xóa file tạm
    let _ = fs::remove_file(&output);

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "application/pdf".parse().unwrap());
    headers.insert(
        CONTENT_DISPOSITION,
        "inline; filename=\"invoice.pdf\"".parse().unwrap(),
    );

    Ok((headers, Body::from(bytes)))
}

pub async fn get_printers() -> impl IntoResponse {
    match printer_core::get_printers() {
        Ok(list) => Json(list).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn prepare_reports(reports: &mut Vec<PdfTemplate>) -> anyhow::Result<()> {
    for report in reports.iter_mut() {
        if let Some(background_image) = report.background_image.as_mut() {
            if background_image.starts_with("http") {
                let url = background_image.clone();
                let base64 = get_image_base64(&url).await.map_err(anyhow::Error::msg)?;
                *background_image = base64;
            }
        }

        for element in report.elements.iter_mut() {
            if let Element::Image(image) = element {
                if let Some(content) = image.content.as_ref() {
                    if content.starts_with("http") {
                        let url = content.clone();
                        let base64 = get_image_base64(&url).await.map_err(anyhow::Error::msg)?;
                        image.content = Some(base64);
                    }
                }
            }
        }
    }

    Ok(())
}
