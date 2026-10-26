use tauri::State;
use vaone_tax_sync::{
    database::Database,
    models::{InvoiceStats, SyncDay, SyncInvoiceFilters, SyncInvoicePage},
};

use crate::services::worker_manage::TaxSyncWorkerManager;

#[tauri::command]
pub fn get_invoice_stats(
    tax_code: String,
    db: tauri::State<'_, Database>,
) -> Result<InvoiceStats, String> {
    db.get_invoice_stats(&tax_code).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sync_days(
    tax_code: String,
    is_success: Option<bool>,
    db: tauri::State<'_, Database>,
) -> std::result::Result<Vec<SyncDay>, String> {
    db.get_sync_days(&tax_code, is_success)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sync_invoices(
    filters: SyncInvoiceFilters,
    page: u32,
    page_size: u32,
    db: tauri::State<'_, Database>,
) -> std::result::Result<SyncInvoicePage, String> {
    db.get_sync_invoices(&filters, page, page_size)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_sync_status(
    tax_code: String,
    from_date: String,
    to_date: String,
    tax_sync: bool,
    app_sync: bool,
    db: tauri::State<'_, Database>,
) -> std::result::Result<(), String> {
    db.reset_sync_status(&tax_code, &from_date, &to_date, tax_sync, app_sync)
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
