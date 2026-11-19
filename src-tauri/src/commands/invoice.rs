use crate::models::system::SyncState;
use crate::progress_bar;
use crate::services::other_invoice::m_invoice::run_sync_flow_m_invoice;
use crate::services::other_invoice::save_invoice::run_sync_flow_save_invoice;
use crate::services::tct::fetch_invoices::run_sync_flow;

#[tauri::command]
pub fn get_sync_state(tenant_id: String) -> Option<SyncState> {
    crate::state::get_sync(&tenant_id)
}

#[tauri::command]
pub async fn start_invoice_tct_sync(
    tenant_id: String,
    org_unit_id: String,
    invoice_type: u8,
    from_date: String,
    to_date: String,
    token: String, // Token của TCT
    delay: u64,
) -> Result<(), String> {
    if !crate::state::try_start_sync(&tenant_id, "TCT") {
        return Ok(());
    }

    crate::state::update_sync_emit(&tenant_id, |s| {
        s.invoice_type = invoice_type;
        s.completed = 0;
        s.failed = 0;
        s.success = 0;
        s.source = "TCT".to_string();
        s.running = true;
        s.current_invoice = None;
        s.total = None;
        s.message.clear();
        s.is_error_api = false;
    });

    tokio::spawn(async move {
        run_sync_flow(
            tenant_id,
            org_unit_id,
            invoice_type,
            from_date,
            to_date,
            Some(token),
            Some(delay),
        )
        .await;
    });

    Ok(())
}

#[tauri::command]
pub async fn start_m_invoice_sync(
    tenant_id: String,
    org_unit_id: String,
    invoice_type: u8,
    from_date: String,
    to_date: String,
    token: String, // Token M-Invoice
    delay: u64,
    tax_code: String,
) -> Result<(), String> {
    if !crate::state::try_start_sync(&tenant_id, "M-SMI") {
        return Ok(());
    }
    crate::state::update_sync_emit(&tenant_id, |s| {
        s.invoice_type = invoice_type;
        s.completed = 0;
        s.failed = 0;
        s.success = 0;
        s.source = "M-SMI".to_string();
        s.running = true;
        s.current_invoice = None;
        s.total = None;
        s.message.clear();
        s.is_error_api = false;
    });

    tokio::spawn(async move {
        run_sync_flow_m_invoice(
            tenant_id,
            org_unit_id,
            invoice_type,
            from_date,
            to_date,
            Some(token),
            Some(delay),
            tax_code,
        )
        .await;
    });
    Ok(())
}

#[tauri::command]
pub async fn start_save_invoice_sync(
    tenant_id: String,
    org_unit_id: String,
    invoice_type: u8,
    from_date: String,
    to_date: String,
    token: String, // Token SAVE-INVOICE
    delay: u64,
    id_account: String,
) -> Result<(), String> {
    if !crate::state::try_start_sync(&tenant_id, "SAVE-INVOICE") {
        return Ok(());
    }

    crate::state::update_sync_emit(&tenant_id, |s| {
        s.invoice_type = invoice_type;
        s.completed = 0;
        s.failed = 0;
        s.success = 0;
        s.source = "SAVE-INVOICE".to_string();
        s.running = true;
        s.current_invoice = None;
        s.total = None;
        s.message.clear();
        s.is_error_api = false;
    });

    tokio::spawn(async move {
        run_sync_flow_save_invoice(
            tenant_id,
            org_unit_id,
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
