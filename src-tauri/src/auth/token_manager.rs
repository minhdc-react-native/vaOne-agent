use crate::models::system::{AppState, AuthConfig, TenantState, TokenState};
use crate::state::APP_STATE;

pub struct TokenManager;

impl TokenManager {
    /// Tạo tenant nếu chưa có
    fn tenant_mut<'a>(state: &'a mut AppState, tenant_id: &str) -> &'a mut TenantState {
        state.tenants.entry(tenant_id.to_string()).or_default()
    }

    /// Web sync token sang tray
    pub fn sync(tenant_id: &str, token: Option<TokenState>, auth: Option<AuthConfig>) {
        let mut state = APP_STATE
            .get()
            .expect("APP_STATE not initialized")
            .lock()
            .unwrap();

        let tenant = Self::tenant_mut(&mut state, tenant_id);

        if let Some(auth) = auth {
            tenant.auth = Some(auth);
        }

        match token {
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
                // logout
                tenant.token = None;
            }
        }
    }

    /// Lấy token
    pub fn get_token(tenant_id: &str) -> Option<TokenState> {
        APP_STATE
            .get()
            .expect("APP_STATE not initialized")
            .lock()
            .unwrap()
            .tenants
            .get(tenant_id)
            .and_then(|t| t.token.clone())
    }

    /// Lấy auth config
    pub fn get_auth(tenant_id: &str) -> Option<AuthConfig> {
        APP_STATE
            .get()
            .expect("APP_STATE not initialized")
            .lock()
            .unwrap()
            .tenants
            .get(tenant_id)
            .and_then(|t| t.auth.clone())
    }

    /// Đánh dấu web online
    pub fn set_web_online(tenant_id: &str, online: bool) {
        let mut state = APP_STATE
            .get()
            .expect("APP_STATE not initialized")
            .lock()
            .unwrap();

        let tenant = Self::tenant_mut(&mut state, tenant_id);

        tenant.web_online = online;
    }

    /// heartbeat
    pub fn heartbeat(tenant_id: &str, timestamp: i64) {
        let mut state = APP_STATE
            .get()
            .expect("APP_STATE not initialized")
            .lock()
            .unwrap();

        let tenant = Self::tenant_mut(&mut state, tenant_id);

        tenant.last_heartbeat = timestamp;
        tenant.web_online = true;
    }
}
