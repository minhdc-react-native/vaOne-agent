use crate::models::agent_info::AgentInfo;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[tauri::command]
pub fn get_agent_info() -> AgentInfo {
    AgentInfo {
        name: "vaOne Agent".into(),
        version: "1.0.0".into(),
        os: std::env::consts::OS.into(),
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
