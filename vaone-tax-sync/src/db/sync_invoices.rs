use rusqlite::params;
use serde_json::Value;

use crate::models::{
    InvoiceStats, InvoiceStatsGroup, InvoiceUpload, SyncDay, SyncInvoice, SyncInvoiceFilters,
    SyncInvoicePage,
};
use crate::{
    database::{db_error, Database},
    error::{Result, TaxSyncError},
};

// ---------------------------------------------------------
// Quản lý tài khoản mã số thuế dành cho UI
// ---------------------------------------------------------

impl Database {
    pub fn get_invoice_stats(&self, tax_code: &str) -> Result<InvoiceStats> {
        let tax_code = tax_code.trim();

        if tax_code.is_empty() {
            return Err(TaxSyncError::Database(
                "Mã số thuế không được để trống".to_string(),
            ));
        }

        let conn = self.connect()?;

        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(
                SELECT 1 FROM tax_accounts WHERE tax_code = ?1
            )",
                params![tax_code],
                |row| row.get(0),
            )
            .map_err(db_error)?;

        if !exists {
            return Err(TaxSyncError::Database(format!(
                "Không tìm thấy tài khoản mã số thuế: {tax_code}"
            )));
        }

        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*)
             FROM sync_invoices
             WHERE tax_code = ?1",
                params![tax_code],
                |row| row.get(0),
            )
            .map_err(db_error)?;

        let mut stmt = conn
            .prepare(
                "SELECT
                invoice_type,
                detail_status,
                upload_status,
                COUNT(*) AS total
             FROM sync_invoices
             WHERE tax_code = ?1
             GROUP BY
                invoice_type,
                detail_status,
                upload_status
             ORDER BY
                invoice_type,
                detail_status,
                upload_status",
            )
            .map_err(db_error)?;

        let rows = stmt
            .query_map(params![tax_code], |row| {
                Ok(InvoiceStatsGroup {
                    invoice_type: row.get(0)?,
                    detail_status: row.get(1)?,
                    upload_status: row.get(2)?,
                    count: row.get(3)?,
                })
            })
            .map_err(db_error)?;

        let mut groups = Vec::new();

        for row in rows {
            groups.push(row.map_err(db_error)?);
        }

        Ok(InvoiceStats {
            tax_code: tax_code.to_string(),
            total,
            groups,
        })
    }

    pub fn get_sync_days(&self, tax_code: &str) -> Result<Vec<SyncDay>> {
        let tax_code = tax_code.trim();

        if tax_code.is_empty() {
            return Err(TaxSyncError::Database(
                "Mã số thuế không được để trống".to_string(),
            ));
        }

        let conn = self.connect()?;

        let mut stmt = conn
            .prepare(
                "SELECT
                tax_code,
                invoice_type,
                sync_date,
                list_status,
                status,
                last_error,
                attempt_count,
                started_at,
                completed_at,
                updated_at
             FROM sync_days
             WHERE tax_code = ?1
             ORDER BY invoice_type ASC, sync_date DESC",
            )
            .map_err(db_error)?;

        let rows = stmt
            .query_map(params![tax_code], |row| {
                Ok(SyncDay {
                    tax_code: row.get(0)?,
                    invoice_type: row.get(1)?,
                    sync_date: row.get(2)?,
                    list_status: row.get(3)?,
                    status: row.get(4)?,
                    last_error: row.get(5)?,
                    attempt_count: row.get(6)?,
                    started_at: row.get(7)?,
                    completed_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(db_error)?;

        let mut items = Vec::new();

        for row in rows {
            items.push(row.map_err(db_error)?);
        }

        Ok(items)
    }

    pub fn get_sync_invoices(
        &self,
        filters: &SyncInvoiceFilters,
        page: u32,
        page_size: u32,
    ) -> Result<SyncInvoicePage> {
        let tax_code = filters
            .tax_code
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        let from_date = filters.from_date.as_deref().filter(|s| !s.is_empty());

        let to_date = filters.to_date.as_deref().filter(|s| !s.is_empty());

        let detail_status = filters.detail_status.as_deref().filter(|s| !s.is_empty());

        let upload_status = filters.upload_status.as_deref().filter(|s| !s.is_empty());

        if let (Some(from), Some(to)) = (from_date, to_date) {
            if from > to {
                return Err(TaxSyncError::Database(
                    "Từ ngày không được lớn hơn đến ngày".to_string(),
                ));
            }
        }

        let page = page.max(1);
        let page_size = page_size.clamp(1, 500);
        let offset = (page as i64 - 1) * page_size as i64;

        let conn = self.connect()?;

        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*)
             FROM sync_invoices
             WHERE (?1 IS NULL OR tax_code = ?1)
               AND (?2 IS NULL OR invoice_type = ?2)
               AND (?3 IS NULL OR invoice_date >= ?3)
               AND (?4 IS NULL OR invoice_date <= ?4)
               AND (?5 IS NULL OR detail_status = ?5)
               AND (?6 IS NULL OR upload_status = ?6)",
                params![
                    tax_code,
                    filters.invoice_type,
                    from_date,
                    to_date,
                    detail_status,
                    upload_status,
                ],
                |row| row.get(0),
            )
            .map_err(db_error)?;

        let mut stmt = conn
            .prepare(
                "SELECT
                tax_code,
                invoice_type,
                invoice_key,
                dvtte,
                tgia,
                nbmst,
                khhdon,
                shdon,
                khmshdon,
                invoice_date,
                tgtcthue,
                tgtthue,
                tgtttbso,
                tthai,
                ttxly,
                detail_status,
                upload_status,
                detail_attempt_count,
                upload_attempt_count,
                detail_error,
                upload_error,
                updated_at,
                completed_at
             FROM sync_invoices
             WHERE (?1 IS NULL OR tax_code = ?1)
               AND (?2 IS NULL OR invoice_type = ?2)
               AND (?3 IS NULL OR invoice_date >= ?3)
               AND (?4 IS NULL OR invoice_date <= ?4)
               AND (?5 IS NULL OR detail_status = ?5)
               AND (?6 IS NULL OR upload_status = ?6)
             ORDER BY invoice_date DESC, invoice_key DESC
             LIMIT ?7 OFFSET ?8",
            )
            .map_err(db_error)?;

        let rows = stmt
            .query_map(
                params![
                    tax_code,
                    filters.invoice_type,
                    from_date,
                    to_date,
                    detail_status,
                    upload_status,
                    page_size,
                    offset,
                ],
                |row| {
                    Ok(SyncInvoice {
                        tax_code: row.get(0)?,
                        invoice_type: row.get(1)?,
                        invoice_key: row.get(2)?,
                        dvtte: row.get(3)?,
                        tgia: row.get(4)?,
                        nbmst: row.get(5)?,
                        khhdon: row.get(6)?,
                        shdon: row.get(7)?,
                        khmshdon: row.get(8)?,
                        invoice_date: row.get(9)?,
                        tgtcthue: row.get(10)?,
                        tgtthue: row.get(11)?,
                        tgtttbso: row.get(12)?,
                        tthai: row.get(13)?,
                        ttxly: row.get(14)?,
                        detail_status: row.get(15)?,
                        upload_status: row.get(16)?,
                        detail_attempt_count: row.get(17)?,
                        upload_attempt_count: row.get(18)?,
                        detail_error: row.get(19)?,
                        upload_error: row.get(20)?,
                        updated_at: row.get(21)?,
                        completed_at: row.get(22)?,
                    })
                },
            )
            .map_err(db_error)?;

        let mut items = Vec::new();

        for row in rows {
            items.push(row.map_err(db_error)?);
        }

        let total_pages = if total == 0 {
            0
        } else {
            ((total - 1) / page_size as i64 + 1) as u32
        };

        Ok(SyncInvoicePage {
            items,
            total,
            page,
            page_size,
            total_pages,
        })
    }

    pub fn get_invoices_pending_upload(&self, tax_code: &str) -> Result<Vec<InvoiceUpload>> {
        let tax_code = tax_code.trim();

        if tax_code.is_empty() {
            return Err(TaxSyncError::Database(
                "Mã số thuế không được để trống".to_string(),
            ));
        }

        let conn = self.connect()?;

        let mut stmt = conn
            .prepare(
                "SELECT
                tax_code,
                invoice_key,
                invoice_type,
                summary_json,
                xml_data
             FROM sync_invoices
             WHERE tax_code = ?1
               AND detail_status = 'completed'
               AND upload_status <> 'completed'
             ORDER BY invoice_type, invoice_key",
            )
            .map_err(db_error)?;

        let rows = stmt
            .query_map(params![tax_code], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u8>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(db_error)?;

        let mut invoices = Vec::new();

        for row in rows {
            let (tax_code, invoice_key, invoice_type, summary_json, xml_data) =
                row.map_err(db_error)?;

            let invoice: Value = serde_json::from_str(&summary_json).map_err(|e| {
                TaxSyncError::Database(format!(
                    "Không thể parse summary_json của hóa đơn {invoice_key}: {e}"
                ))
            })?;

            invoices.push(InvoiceUpload {
                tax_code,
                invoice_key,
                invoice_type,
                invoice,
                xml_data: Some(xml_data),
            });
        }

        Ok(invoices)
    }
}
