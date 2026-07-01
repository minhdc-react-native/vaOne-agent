use serde::{Deserialize, Serialize};

/// Event envelope gửi từ Tray → Web
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEvent<T> {
    pub event: String,
    pub data: T,
}

/// Danh sách event constants
pub struct WsEvents;

impl WsEvents {
    pub const SYNC_STATE: &'static str = "SYNC_STATE";
    pub const JOB_PROGRESS: &'static str = "JOB_PROGRESS";
    pub const JOB_STARTED: &'static str = "JOB_STARTED";
    pub const JOB_FINISHED: &'static str = "JOB_FINISHED";
    pub const TOKEN_UPDATED: &'static str = "TOKEN_UPDATED";
    pub const TRAY_NAVIGATE: &'static str = "TRAY_NAVIGATE";
    pub const ERROR: &'static str = "ERROR";
}
