use vaone_tax_sync::{
    database::Database,
    models::{InvoiceStats, SyncDay, SyncInvoiceFilters, SyncInvoicePage},
};

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
    db: tauri::State<'_, Database>,
) -> std::result::Result<Vec<SyncDay>, String> {
    db.get_sync_days(&tax_code).map_err(|e| e.to_string())
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
