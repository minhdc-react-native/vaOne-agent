use crate::services::other_invoice::save_invoice::run_sync_flow_save_invoice;
use crate::services::tct::fetch_invoices::run_sync_flow;
use crate::state::SyncState;
#[tauri::command]
pub fn get_sync_state() -> SyncState {
    crate::state::get_sync()
}

#[tauri::command]
pub async fn start_invoice_tct_sync(
    invoice_type: u8,
    from_date: String,
    to_date: String,
    token: String,
    delay: u64,
) -> Result<(), String> {
    if !crate::state::try_start_sync("TCT") {
        return Ok(());
    }
    // 1. reset state
    crate::state::update_sync_emit(|s| {
        s.completed = 0;
        s.failed = 0;
        s.source = "TCT".to_string();
        s.running = true;
        s.current_invoice = None;
        s.total = None;
    });

    tokio::spawn(async move {
        run_sync_flow(invoice_type, from_date, to_date, Some(token), Some(delay)).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn start_save_invoice_sync(
    invoice_type: u8,
    from_date: String,
    to_date: String,
    token: String,
    delay: u64,
    id_account: String,
) -> Result<(), String> {
    if !crate::state::try_start_sync("SAVE-INVOICE") {
        return Ok(());
    }
    // 1. reset state
    crate::state::update_sync_emit(|s| {
        s.completed = 0;
        s.failed = 0;
        s.source = "SAVE-INVOICE".to_string();
        s.running = true;
        s.current_invoice = None;
        s.total = None;
    });
    tokio::spawn(async move {
        run_sync_flow_save_invoice(
            invoice_type,
            from_date,
            to_date,
            Some(token),
            Some(delay),
            id_account,
        )
        .await;
    });

    Ok(())
}
