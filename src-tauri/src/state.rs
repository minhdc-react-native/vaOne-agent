use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

// ==========================
// APP HANDLE (Tauri)
// ==========================
pub static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

// ==========================
// FONT STATIC RESOURCE
// ==========================
pub static FONT: &[u8] = include_bytes!("../fonts/NotoSans-Regular.ttf");

// ==========================
// APP STATE (BUSINESS STATE)
// ==========================
pub static APP_STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

#[derive(Debug, Default)]
pub struct AppState {
    pub sync: SyncState,
}

// ==========================
// SYNC STATE (JOB STATUS)
// ==========================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub source: String,
    pub running: bool,
    pub total: usize,
    pub completed: usize,
    pub success: usize,
    pub failed: usize,
    pub current_invoice: Option<serde_json::Value>,
    pub message: String,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            source: String::new(),
            running: false,
            total: 0,
            completed: 0,
            success: 0,
            failed: 0,
            current_invoice: None,
            message: String::new(),
        }
    }
}

// ==========================
// UPDATE SYNC STATE
// ==========================
pub fn update_sync<F>(f: F)
where
    F: FnOnce(&mut SyncState),
{
    if let Some(state) = APP_STATE.get() {
        let mut state = state.lock().unwrap();
        f(&mut state.sync);
    }
}

// ==========================
// GET SYNC STATE
// ==========================
pub fn get_sync() -> SyncState {
    APP_STATE
        .get()
        .expect("APP_STATE not initialized")
        .lock()
        .unwrap()
        .sync
        .clone()
}

// ======================================================
// WEBSOCKET STATE (TRAY → WEB REALTIME LAYER)
// ======================================================

use std::sync::Arc;
use tokio::sync::mpsc;

// Sender channel cho mỗi WebSocket client
pub type WsTx = mpsc::UnboundedSender<String>;

#[derive(Clone)]
pub struct WsState {
    pub clients: Arc<Mutex<Vec<WsTx>>>,
}

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
    pub fn broadcast_json<T: serde::Serialize>(&self, event: &str, data: T) {
        let payload = serde_json::json!({
            "event": event,
            "data": data
        });

        if let Ok(msg) = serde_json::to_string(&payload) {
            self.broadcast(msg);
        }
    }
}

// ==========================
// HELPER: EMIT SYNC STATE
// ==========================
pub fn emit_sync_state() {
    if let Some(ws) = WS_STATE.get() {
        let sync = get_sync();
        ws.broadcast_json("SYNC_STATE", sync);
    }
}

pub fn update_sync_emit<F>(f: F)
where
    F: FnOnce(&mut SyncState),
{
    if let Some(state) = APP_STATE.get() {
        let mut state = state.lock().unwrap();

        f(&mut state.sync);

        if let Some(ws) = WS_STATE.get() {
            ws.broadcast_json("SYNC_STATE", &state.sync);
        }
    }
}

use std::sync::atomic::{AtomicBool, Ordering};
static SYNC_CANCEL: OnceLock<Arc<AtomicBool>> = OnceLock::new();

pub fn get_cancel_flag() -> Arc<AtomicBool> {
    SYNC_CANCEL
        .get_or_init(|| Arc::new(AtomicBool::new(false)))
        .clone()
}
