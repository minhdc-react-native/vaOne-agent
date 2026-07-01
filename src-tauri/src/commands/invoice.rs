use crate::services::tct::fetch_invoices::run_sync_flow;
use crate::state::SyncState;
#[tauri::command]
pub fn get_sync_state() -> SyncState {
    crate::state::get_sync()
}

#[tauri::command]
pub async fn start_invoice_tct_sync(url: String, token: String, delay: u64) -> Result<(), String> {
    tokio::spawn(async move {
        run_sync_flow(url, Some(token), Some(delay)).await;
    });
    Ok(())
}
