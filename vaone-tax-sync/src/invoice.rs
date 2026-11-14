use chrono::{DateTime, FixedOffset};
use serde_json::Value;

use crate::error::{Result, TaxSyncError};

/// Một hóa đơn trong danh sách kết quả API thuế.
#[derive(Debug, Clone)]
pub struct InvoiceSummary {
    pub raw: Value,
    /// Ngày hóa đơn theo giờ Việt Nam: YYYY-MM-DD.
    pub invoice_date: String,
    pub tax_code: String,
    pub invoice_serial: String,
    pub invoice_number: i64,
    pub invoice_type: Option<i64>,
}

impl InvoiceSummary {
    /// Chuẩn hóa dữ liệu danh sách hóa đơn từ API thuế.
    pub fn from_value(raw: Value) -> Result<Self> {
        let raw_date = required_string(&raw, "tdlap")?;
        let invoice_date = normalize_vietnam_date(&raw_date)?;

        let tax_code = required_string(&raw, "nbmst")?;
        let invoice_serial = required_string(&raw, "khhdon")?;

        let invoice_number = raw
            .get("shdon")
            .and_then(as_i64)
            .ok_or(TaxSyncError::MissingField("shdon"))?;

        let invoice_type = raw.get("khmshdon").and_then(as_i64);

        Ok(Self {
            raw,
            invoice_date,
            tax_code,
            invoice_serial,
            invoice_number,
            invoice_type,
        })
    }

    /// Khóa ổn định để lưu vào SQLite.
    /// Ưu tiên UUID id do API thuế cung cấp.
    pub fn invoice_key(&self) -> String {
        if let Some(id) = self.raw.get("id").and_then(Value::as_str) {
            if !id.trim().is_empty() {
                return format!("id:{}", id.trim());
            }
        }

        format!(
            "{}|{}|{}|{}",
            self.tax_code, self.invoice_date, self.invoice_serial, self.invoice_number
        )
    }

    /// Trả về dữ liệu gốc để lưu JSON hoặc ghi log.
    pub fn raw(&self) -> &Value {
        &self.raw
    }
}

fn required_string(value: &Value, field: &'static str) -> Result<String> {
    let item = value.get(field).ok_or(TaxSyncError::MissingField(field))?;

    match item {
        Value::String(s) if !s.trim().is_empty() => Ok(s.clone()),
        Value::Number(n) => Ok(n.to_string()),
        _ => Err(TaxSyncError::InvalidInvoice(format!(
            "Field `{field}` is empty or has an invalid type"
        ))),
    }
}

fn as_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse::<i64>().ok(),
        _ => None,
    }
}

/// Chuyển timestamp RFC3339 về ngày theo múi giờ Việt Nam (UTC+7).
fn normalize_vietnam_date(value: &str) -> Result<String> {
    let parsed = DateTime::parse_from_rfc3339(value).map_err(|_| {
        TaxSyncError::InvalidInvoice(format!(
            "Invalid invoice date `{value}`; expected RFC3339 timestamp"
        ))
    })?;

    let vietnam_offset = FixedOffset::east_opt(7 * 60 * 60)
        .ok_or_else(|| TaxSyncError::InvalidInvoice("Invalid Vietnam timezone".to_string()))?;

    Ok(parsed
        .with_timezone(&vietnam_offset)
        .format("%Y-%m-%d")
        .to_string())
}

/// Xác thực các trường cần thiết để xử lý hóa đơn.
pub fn validate_detail_fields(summary: &InvoiceSummary) -> Result<()> {
    if summary.invoice_date.trim().is_empty() {
        return Err(TaxSyncError::MissingField("tdlap"));
    }

    if summary.tax_code.trim().is_empty() {
        return Err(TaxSyncError::MissingField("nbmst"));
    }

    if summary.invoice_serial.trim().is_empty() {
        return Err(TaxSyncError::MissingField("khhdon"));
    }

    if summary.invoice_number <= 0 {
        return Err(TaxSyncError::InvalidInvoice(
            "Invoice number must be greater than zero".to_string(),
        ));
    }

    Ok(())
}
