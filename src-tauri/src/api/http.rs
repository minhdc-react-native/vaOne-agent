use crate::services::local_server::types::SourceInvoice;
use crate::{
    auth::{auth_api::ensure_valid_token, token_manager::TokenManager},
    state::get_client,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::RngExt;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use serde_json::Value;
use std::{collections::HashMap, time::Duration};
use url::{form_urlencoded, Url};
use uuid::Uuid;
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

    let mut has_request_id = false;

    if let Some(headers) = headers {
        for (k, v) in headers {
            if k.eq_ignore_ascii_case("request-id") {
                has_request_id = true;
            }

            header_map.insert(
                HeaderName::from_bytes(k.as_bytes()).map_err(|e| e.to_string())?,
                HeaderValue::from_str(&v).map_err(|e| e.to_string())?,
            );
        }
    }

    // Tự tạo Request-Id nếu caller chưa truyền
    if !has_request_id {
        header_map.insert(
            HeaderName::from_static("request-id"),
            HeaderValue::from_str(&Uuid::new_v4().to_string()).map_err(|e| e.to_string())?,
        );
    }

    Ok(header_map)
}

pub async fn wait(delay: Option<u64>) {
    let base = delay.unwrap_or(500);

    let duration = rand::rng().random_range(base.saturating_sub(100)..=base.saturating_add(100));

    tokio::time::sleep(std::time::Duration::from_millis(duration)).await;
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

    let request_headers = build_headers(token, headers)?;

    let request = client
        .post(url)
        .headers(request_headers)
        .json(body)
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.execute(request).await.map_err(|e| e.to_string())?;

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

pub async fn post_data(
    tenant_id: &str,
    org_unit_id: &str,
    source: SourceInvoice,
    body: &Value,
) -> ApiResult<Value> {
    let token = ensure_valid_token(tenant_id)
        .await
        .map_err(|e| e.to_string())?;

    // Lấy url từ AuthConfig
    let post_url = TokenManager::get_auth(tenant_id)
        .ok_or_else(|| "Auth config not found".to_string())?
        .post_data_url;

    if post_url.trim().is_empty() {
        wait(Some(1000)).await;
        return Ok(serde_json::json!({
            "success": true,
            "mock": true
        }));
    }

    let client = get_client();

    let new_body = serde_json::json!({
        "source": source,
        "data": body,
    });

    let response = client
        .post(post_url)
        .bearer_auth(&token.access_token)
        .header("__tenant", tenant_id)
        .header("__orgId", org_unit_id)
        .json(&new_body)
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

pub async fn get_image_base64(url: &str) -> Result<String, String> {
    let client = Client::builder().cookie_store(true).build().unwrap();

    let response = client.get(url).send().await.map_err(|e| e.to_string())?;

    let status = response.status();

    if !status.is_success() {
        let text = response.text().await.map_err(|e| e.to_string())?;

        return Err(format!("HTTP {}: {}", status, text));
    }

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .split(';')
        .next()
        .unwrap_or("application/octet-stream")
        .to_string();

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;

    let encoded = STANDARD.encode(&bytes);

    Ok(format!("data:{};base64,{}", content_type, encoded))
}
