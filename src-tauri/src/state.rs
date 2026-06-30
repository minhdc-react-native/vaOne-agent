use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

pub static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

pub static FONT: &[u8] = include_bytes!("../fonts/NotoSans-Regular.ttf");

pub static APP_STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub running: bool,

    /// Tổng số invoice cần xử lý
    pub total: usize,

    /// Đã xử lý
    pub completed: usize,

    /// Thành công
    pub success: usize,

    /// Thất bại
    pub failed: usize,

    /// Invoice hiện tại
    pub current_invoice: Option<String>,

    /// Thông báo cuối
    pub message: String,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
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

#[derive(Debug, Default)]
pub struct AppState {
    pub sync: SyncState,
}

pub fn update_sync<F>(f: F)
where
    F: FnOnce(&mut SyncState),
{
    if let Some(state) = APP_STATE.get() {
        let mut state = state.lock().unwrap();
        f(&mut state.sync);
    }
}

pub fn get_sync() -> SyncState {
    APP_STATE.get().unwrap().lock().unwrap().sync.clone()
}
