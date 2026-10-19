use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct SyncRequest {
    pub tax_code: String,
    /// 1: mua vào, 2: bán ra, 3: mua vào MTT, 4: bán ra MTT
    pub invoice_type: u8,
    /// Ngày bắt đầu và kết thúc, định dạng YYYY-MM-DD
    pub from_date: String,
    pub to_date: String,
    /// Token đăng nhập hóa đơn điện tử
    pub token: String,
    /// Khoảng nghỉ giữa các hóa đơn (milliseconds)
    pub delay_ms: u64,
}

#[derive(Debug)]
pub struct InvoiceDetailRetry {
    pub invoice_key: String,
    pub nbmst: String,
    pub khhdon: String,
    pub shdon: i64,
    pub khmshdon: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncSummary {
    pub total: usize,
    pub completed: usize,
    pub success: usize,
    pub failed: usize,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceProgress {
    pub completed: usize,
    pub total: usize,
    pub invoice_date: String,
    pub invoice_number: i64,
    pub invoice_serial: String,
    pub success: bool,
    pub error: Option<String>,
    pub current_invoice: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct InvoiceSummary {
    pub raw: Value,
}

impl InvoiceSummary {
    pub fn external_fields(&self) -> (String, String, i64, i64) {
        let item = &self.raw;

        (
            item["tdlap"].as_str().unwrap_or("").to_owned(),
            item["nbmst"].as_str().unwrap_or("").to_owned(),
            item["shdon"].as_i64().unwrap_or(0),
            item["khmshdon"].as_i64().unwrap_or(0),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxAccount {
    pub tax_code: String,
    pub username: String,
    pub password_key: String,
    pub start_date: String,
    pub enabled: bool,
    pub invoice_type: String,
    pub auto_sync: bool,
    pub access_code: Option<String>,
    pub type_app: String,
    pub user_app: Option<String>,
    pub pass_app_key: Option<String>,
    pub org_unit: Option<String>,

    pub auth_status: String,
    pub token: Option<String>,
    pub token_expires_at: Option<String>,

    pub app_token: Option<String>,
    pub app_refresh_token: Option<String>,
    pub app_token_expires_at: Option<String>,

    pub last_login_at: Option<String>,
    pub last_auth_error: Option<String>,

    pub last_sync_at: Option<String>,
    pub last_sync_status: String,
    pub last_sync_error: Option<String>,

    pub created_at: Option<String>,
    pub updated_at: Option<String>,

    pub org_unit_id: Option<String>,
    pub tenant_id: Option<String>,
    pub app_token_type: Option<String>,
}

/// Dữ liệu do UI gửi khi thêm mới hoặc sửa tài khoản.
/// Không nhận các trường trạng thái đồng bộ từ UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxAccountInput {
    pub tax_code: String,
    pub username: String,
    pub password_key: String,
    pub start_date: String,
    pub enabled: bool,
    pub invoice_type: String,
    pub auto_sync: bool,
    pub access_code: Option<String>,
    pub type_app: String,
    pub user_app: Option<String>,
    pub pass_app_key: Option<String>,
    pub org_unit: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvoiceStats {
    pub tax_code: String,
    pub total: i64,
    pub groups: Vec<InvoiceStatsGroup>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvoiceStatsGroup {
    pub invoice_type: i64,
    pub detail_status: String,
    pub upload_status: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncDay {
    pub tax_code: String,
    pub invoice_type: i64,
    pub sync_date: String,
    pub list_status: String,
    pub status: String,
    pub last_error: Option<String>,
    pub attempt_count: i64,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub updated_at: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncInvoiceFilters {
    pub tax_code: Option<String>,
    pub invoice_type: Option<i64>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub detail_status: Option<String>,
    pub upload_status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncInvoice {
    pub tax_code: String,
    pub invoice_type: i64,
    pub invoice_key: String,
    pub dvtte: Option<String>,
    pub tgia: Option<f64>,
    pub nbmst: Option<String>,
    pub khhdon: Option<String>,
    pub shdon: Option<i64>,
    pub khmshdon: Option<i64>,
    pub invoice_date: String,
    pub tgtcthue: Option<f64>,
    pub tgtthue: Option<f64>,
    pub tgtttbso: Option<f64>,
    pub tthai: Option<i64>,
    pub ttxly: Option<i64>,
    pub detail_status: String,
    pub upload_status: String,
    pub detail_attempt_count: i64,
    pub upload_attempt_count: i64,
    pub detail_error: Option<String>,
    pub upload_error: Option<String>,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncInvoicePage {
    pub items: Vec<SyncInvoice>,
    pub total: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvoiceUpload {
    pub tax_code: String,
    pub invoice_key: String,
    pub invoice_type: u8,
    pub invoice: Value,
}
