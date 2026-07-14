use std::collections::HashMap;

use serde_json::Value;

use crate::api::http;

pub type ApiResult<T> = Result<T, String>;

#[tauri::command]
pub async fn http_get(
    url: String,
    token: Option<String>,
    delay: Option<u64>,
    headers: Option<HashMap<String, String>>,
    params: Option<HashMap<String, serde_json::Value>>,
) -> ApiResult<Value> {
    http::get(&url, token.as_deref(), delay, headers, params).await
}

#[tauri::command]
pub async fn http_post(
    url: String,
    body: Value,
    token: Option<String>,
    delay: Option<u64>,
    headers: Option<HashMap<String, String>>,
) -> ApiResult<Value> {
    http::post(&url, &body, token.as_deref(), delay, headers).await
}
