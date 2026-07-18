use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncTokenRequest {
    pub tenant_id: String,
    pub token: Option<TokenState>,
    pub auth: AuthConfig,
}

#[derive(Serialize)]
pub struct AgentInfo {
    pub name: String,
    pub version: String,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenState {
    pub tenant_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
    pub version: i64,
    pub leader: TokenLeader,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TokenLeader {
    #[default]
    #[serde(rename = "WEB")]
    WEB,
    #[serde(rename = "TRAY")]
    TRAY,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    pub post_data_url: String,
    pub refresh_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TenantState {
    pub token: Option<TokenState>,
    pub auth: Option<AuthConfig>,
    pub sync: SyncState,
    pub web_online: bool,
    pub last_heartbeat: i64,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub tenants: HashMap<String, TenantState>,
}

// ==========================
// SYNC STATE (JOB STATUS)
// ==========================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub invoice_type: u8,
    pub source: String,
    pub running: bool,
    pub total: Option<usize>,
    pub completed: usize,
    pub success: usize,
    pub failed: usize,
    pub current_invoice: Option<serde_json::Value>,
    pub message: String,
    pub is_error_api: bool,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            invoice_type: 0,
            source: String::new(),
            running: false,
            total: None,
            completed: 0,
            success: 0,
            failed: 0,
            current_invoice: None,
            message: String::new(),
            is_error_api: false,
        }
    }
}
