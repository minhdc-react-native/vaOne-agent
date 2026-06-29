use crate::models::agent_info::AgentInfo;
use crate::state::APP_HANDLE;
use crate::window_config;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use tauri::{LogicalSize, Manager, Size};
use tokio::time::{sleep, Duration};
#[tauri::command]
pub fn get_agent_info() -> AgentInfo {
    AgentInfo {
        name: "vaOne plugin".into(),
        version: "1.0.0".into(),
        os: std::env::consts::OS.into(),
    }
}

#[tauri::command]
pub fn page_ready(name: String, show: Option<bool>) {
    if let Some(app) = APP_HANDLE.get() {
        if let Some(window) = app.get_webview_window("main") {
            let show = show.unwrap_or(true);
            let (width, height) = window_config::get_window_size(&name).unwrap();
            if show {
                let _ = window.set_size(Size::Logical(LogicalSize { width, height }));
                let _ = window.center();
                let _ = window.show();
                let _ = window.set_focus();
            } else {
                let _ = window.hide();
            }
        }
    }
}

#[tauri::command]
pub async fn get_invoice_detail(
    url: String,
    token: String,
    delay: u64,
) -> Result<HashMap<String, Value>, String> {
    let client = Client::new();

    let res = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    // ⏱ delay 1500ms (sau khi request xong)
    sleep(Duration::from_millis(delay)).await;

    let text = res.text().await.map_err(|e| e.to_string())?;

    // 🔥 debug cực quan trọng (bật khi cần)
    // println!("RAW RESPONSE: {}", text);

    let json: HashMap<String, Value> = serde_json::from_str(&text)
        .map_err(|e| format!("JSON parse error: {} | body: {}", e, text))?;

    Ok(json)
}
