use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    api::http,
    models::system::{AuthConfig, TokenLeader, TokenState},
    state::APP_STATE,
};

pub async fn ensure_valid_token(tenant_id: &str) -> Result<TokenState> {
    // ---------------------------------------------------
    // Lấy snapshot hiện tại
    // ---------------------------------------------------
    let (token, auth) = {
        let state = APP_STATE
            .get()
            .expect("APP_STATE not initialized")
            .lock()
            .unwrap();

        let tenant = state
            .tenants
            .get(tenant_id)
            .ok_or_else(|| anyhow!("Tenant not found"))?;

        (
            tenant
                .token
                .clone()
                .ok_or_else(|| anyhow!("Token not found"))?,
            tenant
                .auth
                .clone()
                .ok_or_else(|| anyhow!("Auth config not found"))?,
        )
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;

    // ---------------------------------------------------
    // Token còn hạn > 60s
    // ---------------------------------------------------
    if token.expires_at - now > 60_000 {
        return Ok(token);
    }

    println!("Refreshing token for tenant {}", tenant_id);

    // ---------------------------------------------------
    // Refresh
    // ---------------------------------------------------
    let mut new_token = refresh_token(&token, &auth).await?;

    new_token.version = token.version + 1;
    new_token.leader = TokenLeader::TRAY;

    // ---------------------------------------------------
    // Update APP_STATE
    // ---------------------------------------------------
    {
        let mut state = APP_STATE
            .get()
            .expect("APP_STATE not initialized")
            .lock()
            .unwrap();

        if let Some(tenant) = state.tenants.get_mut(tenant_id) {
            tenant.token = Some(new_token.clone());
        }
    }

    // ---------------------------------------------------
    // Thông báo Web
    // ---------------------------------------------------
    crate::state::broadcast_token_updated(tenant_id, &new_token);

    Ok(new_token)
}

async fn refresh_token(current: &TokenState, auth: &AuthConfig) -> Result<TokenState> {
    let mut body = HashMap::<String, String>::new();

    body.insert("client_id".into(), auth.client_id.clone());
    body.insert("client_secret".into(), auth.client_secret.clone());
    body.insert("grant_type".into(), "refresh_token".into());
    body.insert("refresh_token".into(), current.refresh_token.clone());

    if let Some(scope) = &auth.scope {
        if !scope.is_empty() {
            body.insert("scope".into(), scope.clone());
        }
    }

    let response = http::post_form(&auth.refresh_url, &body, None, None, None)
        .await
        .map_err(anyhow::Error::msg)?;

    let access_token = response["access_token"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing access_token"))?
        .to_string();

    let refresh_token = response["refresh_token"]
        .as_str()
        .unwrap_or(&current.refresh_token)
        .to_string();

    let expires_in = response["expires_in"].as_i64().unwrap_or(3600);

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as i64;

    Ok(TokenState {
        tenant_id: current.tenant_id.clone(),
        access_token,
        refresh_token,
        expires_at: now + expires_in * 1000,
        version: current.version,
        leader: current.leader.clone(),
    })
}
