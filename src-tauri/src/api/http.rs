use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};
use serde_json::Value;
use std::{collections::HashMap, time::Duration};

pub type ApiResult<T> = Result<T, String>;

fn build_headers(
    token: Option<&str>,
    headers: Option<HashMap<String, String>>,
) -> Result<HeaderMap, String> {
    let mut header_map = HeaderMap::new();

    if let Some(token) = token {
        if !token.trim().is_empty() {
            header_map.insert(
                reqwest::header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token)).map_err(|e| e.to_string())?,
            );
        }
    }

    if let Some(headers) = headers {
        for (k, v) in headers {
            header_map.insert(
                HeaderName::from_bytes(k.as_bytes()).map_err(|e| e.to_string())?,
                HeaderValue::from_str(&v).map_err(|e| e.to_string())?,
            );
        }
    }

    Ok(header_map)
}

async fn wait(delay: Option<u64>) {
    if let Some(ms) = delay {
        if ms > 0 {
            tokio::time::sleep(Duration::from_millis(ms)).await;
        }
    }
}

pub async fn get(
    url: &str,
    token: Option<&str>,
    delay: Option<u64>,
    headers: Option<HashMap<String, String>>,
) -> ApiResult<Value> {
    wait(delay).await;

    let client = Client::new();

    let response = client
        .get(url)
        .headers(build_headers(token, headers)?)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();

    let text = response.text().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status, text));
    }

    serde_json::from_str(&text).map_err(|e| e.to_string())
}

pub async fn post(
    url: &str,
    body: &Value,
    token: Option<&str>,
    delay: Option<u64>,
    headers: Option<HashMap<String, String>>,
) -> ApiResult<Value> {
    wait(delay).await;

    let client = Client::new();

    let response = client
        .post(url)
        .headers(build_headers(token, headers)?)
        .json(body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();

    let text = response.text().await.map_err(|e| e.to_string())?;

    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status, text));
    }

    serde_json::from_str(&text).map_err(|e| e.to_string())
}
