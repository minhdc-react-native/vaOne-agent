use tauri::State;
use vaone_tax_sync::{
    database::Database,
    db::invoice_database::InvoiceDatabaseManager,
    models::{InvoiceStats, SyncDay, SyncInvoiceFilters, SyncInvoicePage},
};

use crate::services::worker_manage::TaxSyncWorkerManager;

#[tauri::command]
pub fn get_invoice_stats(
    tax_code: String,
    invoice_db_manager: tauri::State<'_, InvoiceDatabaseManager>,
) -> Result<InvoiceStats, String> {
    let invoice_db = invoice_db_manager
        .get(&tax_code)
        .map_err(|e| e.to_string())?;
    invoice_db
        .get_invoice_stats(&tax_code)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sync_days(
    tax_code: String,
    is_success: Option<bool>,
    invoice_db_manager: tauri::State<'_, InvoiceDatabaseManager>,
) -> std::result::Result<Vec<SyncDay>, String> {
    let invoice_db = invoice_db_manager
        .get(&tax_code)
        .map_err(|e| e.to_string())?;
    invoice_db
        .get_sync_days(&tax_code, is_success)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sync_invoices(
    tax_code: String,
    filters: SyncInvoiceFilters,
    page: u32,
    page_size: u32,
    invoice_db_manager: tauri::State<'_, InvoiceDatabaseManager>,
) -> std::result::Result<SyncInvoicePage, String> {
    let invoice_db = invoice_db_manager
        .get(&tax_code)
        .map_err(|e| e.to_string())?;

    invoice_db
        .get_sync_invoices(&filters, page, page_size)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_sync_status(
    tax_code: String,
    from_date: String,
    to_date: String,
    tax_sync: bool,
    app_sync: bool,
    invoice_db_manager: tauri::State<'_, InvoiceDatabaseManager>,
) -> std::result::Result<(), String> {
    let invoice_db = invoice_db_manager
        .get(&tax_code)
        .map_err(|e| e.to_string())?;
    invoice_db
        .reset_sync_status(&tax_code, &from_date, &to_date, tax_sync, app_sync)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn start_tax_sync_worker(manager: State<'_, TaxSyncWorkerManager>) -> Result<(), String> {
    manager.start()
}

#[tauri::command]
pub async fn stop_tax_sync_worker(manager: State<'_, TaxSyncWorkerManager>) -> Result<(), String> {
    manager.stop_and_wait().await
}

#[tauri::command]
pub fn is_tax_sync_worker_running(manager: State<'_, TaxSyncWorkerManager>) -> bool {
    manager.is_running()
}
