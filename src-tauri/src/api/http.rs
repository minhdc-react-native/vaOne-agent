use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;
use std::{collections::HashMap, time::Duration};
use url::{form_urlencoded, Url};

use crate::{
    auth::{auth_api::ensure_valid_token, token_manager::TokenManager},
    state::{get_client, APP_STATE},
};

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
    params: Option<HashMap<String, serde_json::Value>>,
) -> ApiResult<Value> {
    wait(delay).await;

    let client = get_client();

    let mut parsed = Url::parse(url).map_err(|e| e.to_string())?;

    if let Some(params) = params {
        {
            let mut pairs = parsed.query_pairs_mut();

            for (k, v) in params {
                let value = match v {
                    serde_json::Value::String(s) => s,
                    _ => v.to_string(),
                };
                pairs.append_pair(&k, &value);
            }
        } // query_pairs_mut kết thúc ở đây
    }
    // println!("url={:#?}", parsed.as_str());
    let response = client
        .get(parsed.as_str())
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

    let client = get_client();

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

pub async fn post_form(
    url: &str,
    form: &HashMap<String, String>,
    token: Option<&str>,
    delay: Option<u64>,
    headers: Option<HashMap<String, String>>,
) -> ApiResult<Value> {
    wait(delay).await;

    let client = get_client();

    let body = form_urlencoded::Serializer::new(String::new())
        .extend_pairs(form.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .finish();

    let response = client
        .post(url)
        .headers(build_headers(token, headers)?)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
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

pub async fn post_data(tenant_id: &str, org_unit_id: &str, body: &Value) -> ApiResult<Value> {
    let token = ensure_valid_token(tenant_id)
        .await
        .map_err(|e| e.to_string())?;

    // Lấy url từ AuthConfig
    let post_url = TokenManager::get_auth(tenant_id)
        .ok_or_else(|| "Auth config not found".to_string())?
        .post_data_url;

    if post_url.trim().is_empty() {
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        return Ok(serde_json::json!({
            "success": true,
            "mock": true
        }));
    }

    let client = get_client();

    let response = client
        .post(post_url)
        .bearer_auth(&token.access_token)
        .header("__tenant", tenant_id)
        .header("__orgId", org_unit_id)
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
