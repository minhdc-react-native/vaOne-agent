use crate::state::SyncState;
#[tauri::command]
pub fn get_sync_state() -> SyncState {
    crate::state::get_sync()
}
