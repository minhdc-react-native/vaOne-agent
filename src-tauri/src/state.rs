use std::sync::{Mutex, OnceLock};

use reqwest::Client;
use tauri::{menu::MenuItem, AppHandle, Wry};

pub static CURRENT_ROUTE: OnceLock<Mutex<String>> = OnceLock::new();

// ==========================
// APP HANDLE (Tauri)
// ==========================
pub static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

// ==========================
// APP STATE (BUSINESS STATE)
// ==========================
pub static APP_STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

pub static ONLINE_MENU: OnceLock<MenuItem<Wry>> = OnceLock::new();

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

pub fn get_client() -> &'static Client {
    HTTP_CLIENT.get_or_init(|| Client::builder().cookie_store(true).build().unwrap())
}

// ==========================
// UPDATE SYNC STATE
// ==========================
pub fn update_sync<F>(tenant_id: &str, f: F)
where
    F: FnOnce(&mut SyncState),
{
    if let Some(state) = APP_STATE.get() {
        let mut state = state.lock().unwrap();

        let tenant = state.tenants.entry(tenant_id.to_string()).or_default();

        f(&mut tenant.sync);
    }
}

pub fn try_start_sync(tenant_id: &str, source: &str) -> bool {
    let mut state = APP_STATE
        .get()
        .expect("APP_STATE not initialized")
        .lock()
        .unwrap();

    let tenant = state.tenants.entry(tenant_id.to_string()).or_default();

    if tenant.sync.running {
        return false;
    }

    tenant.sync.running = true;
    tenant.sync.source = source.to_string();
    tenant.sync.completed = 0;
    tenant.sync.failed = 0;
    tenant.sync.success = 0;
    tenant.sync.total = None;
    tenant.sync.current_invoice = None;
    tenant.sync.message.clear();
    tenant.sync.is_error_api = false;

    if let Some(ws) = WS_STATE.get() {
        ws.broadcast_json("SYNC_STATE", tenant_id, &tenant.sync);
    }

    true
}

// ==========================
// GET SYNC STATE
// ==========================
pub fn get_sync(tenant_id: &str) -> Option<SyncState> {
    APP_STATE
        .get()
        .expect("APP_STATE not initialized")
        .lock()
        .unwrap()
        .tenants
        .get(tenant_id)
        .map(|t| t.sync.clone())
}

// ======================================================
// WEBSOCKET STATE (TRAY → WEB REALTIME LAYER)
// ======================================================

use std::sync::Arc;
use tokio::sync::mpsc;

// Sender channel cho mỗi WebSocket client
pub type WsTx = mpsc::UnboundedSender<String>;

// ==========================
// GLOBAL WS STATE
// ==========================
pub static WS_STATE: OnceLock<WsState> = OnceLock::new();

// ==========================
// INIT WS STATE (CALL IN MAIN)
// ==========================
pub fn init_ws_state() {
    let state = WsState {
        clients: Arc::new(Mutex::new(Vec::new())),
    };

    let _ = WS_STATE.set(state);
}

// ==========================
// WS STATE IMPLEMENTATION
// ==========================
#[derive(Clone)]
pub struct WsState {
    pub clients: Arc<Mutex<Vec<WsTx>>>,
}

impl WsState {
    /// thêm client khi connect
    pub fn add_client(&self, tx: WsTx) {
        let mut clients = self.clients.lock().unwrap();
        clients.push(tx);
    }

    /// broadcast raw string
    pub fn broadcast(&self, message: String) {
        let mut clients = self.clients.lock().unwrap();

        let mut alive = Vec::new();

        for client in clients.iter() {
            if client.send(message.clone()).is_ok() {
                alive.push(client.clone());
            }
        }

        *clients = alive;
    }

    /// broadcast JSON event chuẩn
    pub fn broadcast_json<T: serde::Serialize>(&self, event: &str, tenant_id: &str, data: T) {
        let payload = serde_json::json!({
            "event": event,
            "tenantId": tenant_id,
            "data": data
        });

        if let Ok(msg) = serde_json::to_string(&payload) {
            self.broadcast(msg);
        }
    }

    // /// broadcast event không thuộc tenant (update app, shutdown...)
    // pub fn broadcast_global<T: serde::Serialize>(&self, event: &str, data: T) {
    //     let payload = serde_json::json!({
    //         "event": event,
    //         "data": data
    //     });

    //     if let Ok(msg) = serde_json::to_string(&payload) {
    //         self.broadcast(msg);
    //     }
    // }
}

pub fn broadcast_token_updated(tenant_id: &str, token: &TokenState) {
    if let Some(ws) = WS_STATE.get() {
        let payload = serde_json::json!({
            "tenantId": tenant_id,
            "data": token
        });
        ws.broadcast_json("TOKEN_UPDATED", tenant_id, payload);
    }
}

// ==========================
// HELPER: EMIT SYNC STATE
// ==========================
pub fn emit_sync_state(tenant_id: &str) {
    if let Some(ws) = WS_STATE.get() {
        if let Some(sync) = get_sync(tenant_id) {
            ws.broadcast_json("SYNC_STATE", tenant_id, &sync);
        }
    }
}
// ==========================
// UPDATE SYNC + EMIT
// ==========================
pub fn update_sync_emit<F>(tenant_id: &str, f: F)
where
    F: FnOnce(&mut SyncState),
{
    if let Some(state) = APP_STATE.get() {
        let mut state = state.lock().unwrap();

        let tenant = state.tenants.entry(tenant_id.to_string()).or_default();

        f(&mut tenant.sync);

        if let Some(ws) = WS_STATE.get() {
            ws.broadcast_json("SYNC_STATE", tenant_id, &tenant.sync);
        }
    }
}
use std::sync::atomic::AtomicBool;

use crate::models::system::{AppState, SyncState, TokenState};
static SYNC_CANCEL: OnceLock<Arc<AtomicBool>> = OnceLock::new();

pub fn get_cancel_flag() -> Arc<AtomicBool> {
    SYNC_CANCEL
        .get_or_init(|| Arc::new(AtomicBool::new(false)))
        .clone()
}
