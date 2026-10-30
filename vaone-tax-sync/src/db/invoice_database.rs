use crate::{
    error::{Result, TaxSyncError},
    models::{
        InvoiceDetailRetry, InvoiceStats, InvoiceStatsGroup, InvoiceUpload, SyncDay, SyncInvoice,
        SyncInvoiceFilters, SyncInvoicePage,
    },
};
use chrono::{DateTime, FixedOffset};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const STATUS_COMPLETED: &str = "completed";

/// Hóa đơn cần tiếp tục xử lý sau khi ứng dụng khởi động lại.
#[derive(Debug, Clone)]
pub struct PendingInvoice {
    pub tax_code: String,
    pub invoice_type: u8,
    pub invoice_key: String,
    pub invoice_date: String,
    pub summary_json: String,
    pub xml_data: Option<String>,
    pub detail_status: String,
    pub upload_status: String,
    pub attempt_count: i64,
    pub detail_error: Option<String>,
}

#[derive(Clone)]
pub struct InvoiceDatabaseManager {
    pub base_dir: PathBuf,
}

impl InvoiceDatabaseManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn get(&self, tax_code: &str) -> Result<InvoiceDatabase> {
        InvoiceDatabase::for_tax_code(&self.base_dir, tax_code)
    }

    pub fn initialize_tax_code(&self, tax_code: &str) -> Result<()> {
        let _db = self.get(tax_code)?;
        Ok(())
    }

    pub fn delete_tax_code(&self, tax_code: &str) -> Result<()> {
        let tax_code = tax_code.trim();

        if tax_code.is_empty() || !tax_code.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return Err(TaxSyncError::Database(
                "Mã số thuế không hợp lệ".to_string(),
            ));
        }

        let path = self.base_dir.join(format!("{tax_code}.sqlite3"));

        if path.exists() {
            fs::remove_file(&path).map_err(|e| TaxSyncError::Database(e.to_string()))?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct InvoiceDatabase {
    path: PathBuf,
    tax_code: String,
}

impl InvoiceDatabase {
    /// Tạo database hóa đơn riêng cho một mã số thuế.
    /// Ví dụ: app-data/invoices/0101234567.sqlite3
    pub fn for_tax_code(base_dir: impl AsRef<Path>, tax_code: &str) -> Result<Self> {
        let tax_code = tax_code.trim();

        // Chỉ cho phép ký tự thường dùng trong mã số thuế, tránh path traversal.
        if tax_code.is_empty() || !tax_code.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
            return Err(TaxSyncError::Database(format!(
                "Mã số thuế không hợp lệ để tạo database hóa đơn: {tax_code}"
            )));
        }

        let base_dir = base_dir.as_ref();
        std::fs::create_dir_all(base_dir).map_err(|e| {
            TaxSyncError::Database(format!("Không thể tạo thư mục database hóa đơn: {e}"))
        })?;

        let db = Self {
            path: base_dir.join(format!("{tax_code}.sqlite3")),
            tax_code: tax_code.to_string(),
        };

        db.initialize()?;
        Ok(db)
    }

    pub fn open_existing(base_dir: impl AsRef<Path>, tax_code: &str) -> Result<Self> {
        let tax_code = validate_tax_code(tax_code)?;

        let base_dir = base_dir.as_ref();
        let path = base_dir.join(format!("{tax_code}.sqlite3"));

        println!("Invoice DB base_dir: {:?}", base_dir);
        println!("Invoice DB full path: {:?}", path);
        println!("Invoice DB exists: {}", path.is_file());

        if !path.is_file() {
            return Err(TaxSyncError::Database(format!(
                "Không tìm thấy database hóa đơn của mã số thuế: {tax_code}"
            )));
        }

        Ok(Self { path, tax_code })
    }

    /// Có thể dùng khi cần mở trực tiếp một file database đã biết.
    pub fn new(path: impl AsRef<Path>, tax_code: &str) -> Result<Self> {
        let tax_code = tax_code.trim();
        if tax_code.is_empty() || !tax_code.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
            return Err(TaxSyncError::Database(format!(
                "Mã số thuế không hợp lệ: {tax_code}"
            )));
        }

        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| TaxSyncError::Database(e.to_string()))?;
        }

        let db = Self {
            path,
            tax_code: tax_code.to_string(),
        };
        db.initialize()?;
        Ok(db)
    }

    pub fn tax_code(&self) -> &str {
        &self.tax_code
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn connect(&self) -> Result<Connection> {
        let conn =
            Connection::open(&self.path).map_err(|e| TaxSyncError::Database(e.to_string()))?;

        conn.busy_timeout(std::time::Duration::from_secs(10))
            .map_err(db_error)?;

        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;",
        )
        .map_err(db_error)?;

        Ok(conn)
    }

    /// Tạo các bảng và index cần thiết cho database của mã số thuế này.
    pub fn initialize(&self) -> Result<()> {
        let conn = self.connect()?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sync_days (
                tax_code TEXT NOT NULL,
                invoice_type INTEGER NOT NULL
                    CHECK (invoice_type IN (1, 2, 3, 4)),
                sync_date TEXT NOT NULL,

                list_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (list_status IN ('pending', 'running', 'completed', 'failed')),
                status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'running', 'completed', 'failed')),

                last_error TEXT,
                attempt_count INTEGER NOT NULL DEFAULT 0,
                started_at TEXT,
                completed_at TEXT,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

                PRIMARY KEY (tax_code, invoice_type, sync_date)
            );

            CREATE TABLE IF NOT EXISTS sync_invoices (
                tax_code TEXT NOT NULL,
                invoice_type INTEGER NOT NULL
                    CHECK (invoice_type IN (1, 2, 3, 4)),
                invoice_key TEXT NOT NULL,

                dvtte TEXT,
                tgia REAL,
                nbmst TEXT,
                khhdon TEXT,
                shdon INTEGER,
                khmshdon INTEGER,
                invoice_date TEXT NOT NULL,

                nbnmmst TEXT,
                nbnmten TEXT,
                tgtcthue REAL,
                tgtthue REAL,
                tgtttbso REAL,
                tthai INTEGER,
                ttxly INTEGER,

                summary_json TEXT NOT NULL,
                xml_data TEXT,

                detail_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (detail_status IN ('pending', 'running', 'completed', 'failed')),
                upload_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (upload_status IN ('pending', 'running', 'completed', 'failed')),

                detail_attempt_count INTEGER NOT NULL DEFAULT 0,
                upload_attempt_count INTEGER NOT NULL DEFAULT 0,
                detail_error TEXT,
                upload_error TEXT,

                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                completed_at TEXT,

                PRIMARY KEY (tax_code, invoice_type, invoice_key),

                FOREIGN KEY (tax_code, invoice_type, invoice_date)
                    REFERENCES sync_days (tax_code, invoice_type, sync_date)
                    ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS sync_invoices_today (
                tax_code TEXT NOT NULL,
                invoice_type INTEGER NOT NULL
                    CHECK (invoice_type IN (1, 2, 3, 4)),
                invoice_key TEXT NOT NULL,
                invoice_date TEXT NOT NULL,
                PRIMARY KEY (tax_code, invoice_type, invoice_key)
            );

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_today_date
                ON sync_invoices_today (tax_code, invoice_type, invoice_date);

            CREATE INDEX IF NOT EXISTS idx_sync_days_status
                ON sync_days (status, sync_date);

            CREATE INDEX IF NOT EXISTS idx_sync_days_account_status
                ON sync_days (tax_code, status, sync_date);

            CREATE INDEX IF NOT EXISTS idx_sync_days_tax_type_date
                ON sync_days (tax_code, invoice_type, sync_date DESC);

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_retry
                ON sync_invoices (
                    tax_code, invoice_type, invoice_date, detail_status, upload_status
                );

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_detail_retry
                ON sync_invoices (tax_code, invoice_type, detail_status, updated_at);

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_tax_date
                ON sync_invoices (tax_code, invoice_date DESC);

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_tax_type_date
                ON sync_invoices (tax_code, invoice_type, invoice_date DESC);

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_tax_detail_status
                ON sync_invoices (tax_code, detail_status);

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_tax_upload_status
                ON sync_invoices (tax_code, upload_status);

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_pending_details
                ON sync_invoices (detail_status)
                WHERE detail_status <> 'completed';

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_pending_upload
                ON sync_invoices (upload_status)
                WHERE upload_status <> 'completed';
            "#,
        )
        .map_err(db_error)?;

        Ok(())
    }

    fn check_tax_code(&self, tax_code: &str) -> Result<()> {
        if tax_code != self.tax_code {
            return Err(TaxSyncError::Database(format!(
                "Tax code truyền vào ({tax_code}) không khớp database hiện tại ({})",
                self.tax_code
            )));
        }
        Ok(())
    }

    // ---------------------------------------------------------------------
    // 1. Trạng thái đồng bộ từng ngày
    // ---------------------------------------------------------------------

    pub fn ensure_sync_day(&self, tax_code: &str, invoice_type: u8, sync_date: &str) -> Result<()> {
        self.check_tax_code(tax_code)?;
        validate_invoice_type(invoice_type)?;
        validate_date(sync_date)?;

        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO sync_days (tax_code, invoice_type, sync_date)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(tax_code, invoice_type, sync_date) DO NOTHING",
            params![tax_code, invoice_type, sync_date],
        )
        .map_err(db_error)?;
        Ok(())
    }

    pub fn set_day_running(&self, tax_code: &str, invoice_type: u8, sync_date: &str) -> Result<()> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        conn.execute(
            "UPDATE sync_days
             SET list_status = 'running',
                 status = 'running',
                 last_error = NULL,
                 attempt_count = attempt_count + 1,
                 started_at = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE tax_code = ?1 AND invoice_type = ?2 AND sync_date = ?3",
            params![tax_code, invoice_type, sync_date],
        )
        .map_err(db_error)?;
        Ok(())
    }

    pub fn set_day_pending(&self, tax_code: &str, invoice_type: u8, sync_date: &str) -> Result<()> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        conn.execute(
            "UPDATE sync_days
             SET list_status = 'pending',
                 status = 'pending',
                 last_error = NULL,
                 attempt_count = attempt_count + 1,
                 started_at = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE tax_code = ?1 AND invoice_type = ?2 AND sync_date = ?3",
            params![tax_code, invoice_type, sync_date],
        )
        .map_err(db_error)?;
        Ok(())
    }

    pub fn set_day_list_failed(
        &self,
        tax_code: &str,
        invoice_type: u8,
        sync_date: &str,
        error: &str,
    ) -> Result<()> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        conn.execute(
            "UPDATE sync_days
             SET list_status = 'failed', status = 'failed',
                 last_error = ?4, updated_at = CURRENT_TIMESTAMP
             WHERE tax_code = ?1 AND invoice_type = ?2 AND sync_date = ?3",
            params![tax_code, invoice_type, sync_date, error],
        )
        .map_err(db_error)?;
        Ok(())
    }

    pub fn mark_day_list_completed(
        &self,
        tax_code: &str,
        invoice_type: u8,
        sync_date: &str,
        is_today: &bool,
    ) -> Result<()> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        let list_status = if *is_today { "pending" } else { "completed" };
        conn.execute(
            "UPDATE sync_days
             SET list_status = ?1, last_error = NULL, updated_at = CURRENT_TIMESTAMP
             WHERE tax_code = ?2 AND invoice_type = ?3 AND sync_date = ?4",
            params![list_status, tax_code, invoice_type, sync_date],
        )
        .map_err(db_error)?;
        Ok(())
    }

    pub fn set_day_failed(
        &self,
        tax_code: &str,
        invoice_type: u8,
        sync_date: &str,
        error: &str,
    ) -> Result<()> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        conn.execute(
            "UPDATE sync_days
             SET status = 'failed', last_error = ?4, updated_at = CURRENT_TIMESTAMP
             WHERE tax_code = ?1 AND invoice_type = ?2 AND sync_date = ?3",
            params![tax_code, invoice_type, sync_date, error],
        )
        .map_err(db_error)?;
        Ok(())
    }

    pub fn get_incomplete_days(&self, tax_code: &str, invoice_type: u8) -> Result<Vec<String>> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                "SELECT sync_date FROM sync_days
                 WHERE tax_code = ?1 AND invoice_type = ?2
                   AND list_status != 'completed'
                 ORDER BY sync_date ASC",
            )
            .map_err(db_error)?;

        let rows = stmt
            .query_map(params![tax_code, invoice_type], |row| row.get(0))
            .map_err(db_error)?;

        let mut dates = Vec::new();
        for row in rows {
            dates.push(row.map_err(db_error)?);
        }
        Ok(dates)
    }

    // ---------------------------------------------------------------------
    // 2. Lưu danh sách hóa đơn
    // ---------------------------------------------------------------------

    pub fn save_invoice_summary(
        &self,
        tax_code: &str,
        invoice_type: u8,
        summary: &Value,
        is_today: bool,
    ) -> Result<String> {
        self.check_tax_code(tax_code)?;
        validate_invoice_type(invoice_type)?;

        let invoice_date = invoice_date_vietnam(summary)?;
        let invoice_key = invoice_key(summary)?;
        let summary_json = serde_json::to_string(summary)
            .map_err(|e| TaxSyncError::InvalidInvoice(e.to_string()))?;

        let (nbnmmst, nbnmten) = if invoice_type == 1 || invoice_type == 3 {
            (summary["nbmst"].as_str(), summary["nbten"].as_str())
        } else {
            (summary["nmmst"].as_str(), summary["nmten"].as_str())
        };

        let mut conn = self.connect()?;
        let tx = conn.transaction().map_err(db_error)?;

        tx.execute(
            "INSERT INTO sync_invoices (
                tax_code, invoice_type, invoice_key, invoice_date,
                dvtte, tgia, nbmst, khhdon, shdon, khmshdon,
                tgtcthue, tgtthue, tgtttbso, tthai, ttxly, nbnmmst, nbnmten,
                summary_json, detail_status, upload_status
             )
             VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, 'pending', 'pending'
             )
             ON CONFLICT(tax_code, invoice_type, invoice_key)
             DO UPDATE SET
                invoice_date = excluded.invoice_date,
                dvtte = excluded.dvtte,
                tgia = excluded.tgia,
                nbmst = excluded.nbmst,
                khhdon = excluded.khhdon,
                shdon = excluded.shdon,
                khmshdon = excluded.khmshdon,
                tgtcthue = excluded.tgtcthue,
                tgtthue = excluded.tgtthue,
                tgtttbso = excluded.tgtttbso,
                tthai = excluded.tthai,
                ttxly = excluded.ttxly,
                nbnmmst = excluded.nbnmmst,
                nbnmten = excluded.nbnmten,
                summary_json = excluded.summary_json,
                updated_at = CURRENT_TIMESTAMP",
            params![
                tax_code,
                invoice_type,
                invoice_key,
                invoice_date,
                summary["dvtte"].as_str(),
                summary["tgia"].as_f64(),
                summary["nbmst"].as_str(),
                summary["khhdon"].as_str(),
                summary["shdon"].as_i64(),
                summary["khmshdon"].as_i64(),
                summary["tgtcthue"].as_f64(),
                summary["tgtthue"].as_f64(),
                summary["tgtttbso"].as_f64(),
                summary["tthai"].as_i64(),
                summary["ttxly"].as_i64(),
                nbnmmst,
                nbnmten,
                summary_json,
            ],
        )
        .map_err(db_error)?;

        if is_today {
            tx.execute(
                "INSERT OR IGNORE INTO sync_invoices_today
                 (tax_code, invoice_type, invoice_key, invoice_date)
                 VALUES (?1, ?2, ?3, ?4)",
                params![tax_code, invoice_type, invoice_key, invoice_date],
            )
            .map_err(db_error)?;
        }

        tx.commit().map_err(db_error)?;
        Ok(invoice_key)
    }

    // ---------------------------------------------------------------------
    // 3. Chi tiết hóa đơn
    // ---------------------------------------------------------------------

    pub fn get_invoices_pending_detail(
        &self,
        tax_code: &str,
        invoice_type: u8,
        limit: u32,
    ) -> Result<Vec<InvoiceDetailRetry>> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                "SELECT invoice_key, nbmst, khhdon, shdon, khmshdon
                 FROM sync_invoices
                 WHERE tax_code = ?1 AND invoice_type = ?2
                   AND detail_status IN ('pending', 'failed')
                   AND nbmst IS NOT NULL AND khhdon IS NOT NULL
                   AND shdon IS NOT NULL AND khmshdon IS NOT NULL
                 ORDER BY updated_at ASC LIMIT ?3",
            )
            .map_err(db_error)?;

        let rows = stmt
            .query_map(params![tax_code, invoice_type, limit], |row| {
                Ok(InvoiceDetailRetry {
                    invoice_key: row.get(0)?,
                    nbmst: row.get(1)?,
                    khhdon: row.get(2)?,
                    shdon: row.get(3)?,
                    khmshdon: row.get(4)?,
                })
            })
            .map_err(db_error)?;

        let mut invoices = Vec::new();
        for row in rows {
            invoices.push(row.map_err(db_error)?);
        }
        Ok(invoices)
    }

    pub fn save_invoice_detail(
        &self,
        tax_code: &str,
        invoice_type: u8,
        invoice_key: &str,
        detail: &Option<Value>,
        xml: &Option<String>,
    ) -> Result<()> {
        self.check_tax_code(tax_code)?;
        let detail_data = detail
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| TaxSyncError::InvalidInvoice(e.to_string()))?;

        let conn = self.connect()?;
        conn.execute(
            "UPDATE sync_invoices
             SET summary_json = CASE WHEN ?4 IS NOT NULL THEN ?4 ELSE summary_json END,
                 xml_data = ?5,
                 detail_status = 'completed',
                 detail_error = NULL,
                 updated_at = CURRENT_TIMESTAMP
             WHERE tax_code = ?1 AND invoice_type = ?2 AND invoice_key = ?3",
            params![tax_code, invoice_type, invoice_key, detail_data, xml],
        )
        .map_err(db_error)?;
        Ok(())
    }

    pub fn set_detail_failed(
        &self,
        tax_code: &str,
        invoice_type: u8,
        invoice_key: &str,
        error: &str,
    ) -> Result<()> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        conn.execute(
            "UPDATE sync_invoices
             SET detail_status = 'failed',
                 detail_attempt_count = detail_attempt_count + 1,
                 detail_error = ?4,
                 updated_at = CURRENT_TIMESTAMP
             WHERE tax_code = ?1 AND invoice_type = ?2 AND invoice_key = ?3",
            params![tax_code, invoice_type, invoice_key, error],
        )
        .map_err(db_error)?;
        Ok(())
    }

    // ---------------------------------------------------------------------
    // 4. Upload hóa đơn
    // ---------------------------------------------------------------------

    pub fn set_upload_completed(
        &self,
        tax_code: &str,
        invoice_type: u8,
        invoice_key: &str,
    ) -> Result<()> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        conn.execute(
            "UPDATE sync_invoices
             SET upload_status = 'completed',
                 upload_error = NULL,
                 updated_at = CURRENT_TIMESTAMP
             WHERE tax_code = ?1 AND invoice_type = ?2 AND invoice_key = ?3
               AND detail_status = 'completed' AND xml_data IS NOT NULL",
            params![tax_code, invoice_type, invoice_key],
        )
        .map_err(db_error)?;
        Ok(())
    }

    pub fn set_upload_failed(
        &self,
        tax_code: &str,
        invoice_type: u8,
        invoice_key: &str,
        error: &str,
    ) -> Result<()> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        conn.execute(
            "UPDATE sync_invoices
             SET upload_status = 'failed',
                 upload_attempt_count = upload_attempt_count + 1,
                 upload_error = ?4,
                 updated_at = CURRENT_TIMESTAMP
             WHERE tax_code = ?1 AND invoice_type = ?2 AND invoice_key = ?3",
            params![tax_code, invoice_type, invoice_key, error],
        )
        .map_err(db_error)?;
        Ok(())
    }

    // ---------------------------------------------------------------------
    // 5. Truy vấn hóa đơn cần xử lý lại
    // ---------------------------------------------------------------------

    pub fn get_pending_invoices(
        &self,
        tax_code: &str,
        invoice_type: u8,
        sync_date: &str,
    ) -> Result<Vec<PendingInvoice>> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                "SELECT invoice_key, invoice_date, summary_json, xml_data,
                        detail_status, upload_status, detail_attempt_count, detail_error
                 FROM sync_invoices
                 WHERE tax_code = ?1 AND invoice_type = ?2 AND invoice_date = ?3
                   AND (detail_status != 'completed' OR upload_status != 'completed')
                 ORDER BY invoice_key",
            )
            .map_err(db_error)?;

        let rows = stmt
            .query_map(params![tax_code, invoice_type, sync_date], |row| {
                Ok(PendingInvoice {
                    tax_code: tax_code.to_string(),
                    invoice_type,
                    invoice_key: row.get(0)?,
                    invoice_date: row.get(1)?,
                    summary_json: row.get(2)?,
                    xml_data: row.get(3)?,
                    detail_status: row.get(4)?,
                    upload_status: row.get(5)?,
                    attempt_count: row.get(6)?,
                    detail_error: row.get(7)?,
                })
            })
            .map_err(db_error)?;

        let mut invoices = Vec::new();
        for row in rows {
            invoices.push(row.map_err(db_error)?);
        }
        Ok(invoices)
    }

    pub fn refresh_day_status(
        &self,
        tax_code: &str,
        invoice_type: u8,
        sync_date: &str,
    ) -> Result<bool> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;

        let list_status: Option<String> = conn
            .query_row(
                "SELECT list_status FROM sync_days
                 WHERE tax_code = ?1 AND invoice_type = ?2 AND sync_date = ?3",
                params![tax_code, invoice_type, sync_date],
                |row| row.get(0),
            )
            .optional()
            .map_err(db_error)?;

        let Some(list_status) = list_status else {
            return Ok(false);
        };

        if list_status != STATUS_COMPLETED {
            return Ok(false);
        }

        let pending_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sync_invoices
                 WHERE tax_code = ?1 AND invoice_type = ?2 AND invoice_date = ?3
                   AND (detail_status != 'completed' OR upload_status != 'completed')",
                params![tax_code, invoice_type, sync_date],
                |row| row.get(0),
            )
            .map_err(db_error)?;

        if pending_count > 0 {
            conn.execute(
                "UPDATE sync_days
                 SET status = 'failed',
                     last_error = 'Some invoices are still pending or failed',
                     updated_at = CURRENT_TIMESTAMP
                 WHERE tax_code = ?1 AND invoice_type = ?2 AND sync_date = ?3",
                params![tax_code, invoice_type, sync_date],
            )
            .map_err(db_error)?;
            return Ok(false);
        }

        conn.execute(
            "UPDATE sync_days
             SET status = 'completed', last_error = NULL,
                 completed_at = CURRENT_TIMESTAMP,
                 updated_at = CURRENT_TIMESTAMP
             WHERE tax_code = ?1 AND invoice_type = ?2 AND sync_date = ?3",
            params![tax_code, invoice_type, sync_date],
        )
        .map_err(db_error)?;

        Ok(true)
    }

    /// Kiểm tra database còn hóa đơn chưa hoàn tất bước lấy chi tiết hay không.
    pub fn has_pending_updates(&self, tax_code: &str) -> Result<bool> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM sync_invoices
                    WHERE tax_code = ?1 AND detail_status != 'completed'
                 )",
                params![tax_code],
                |row| row.get(0),
            )
            .map_err(db_error)?;
        Ok(exists)
    }

    /// Kiểm tra database còn hóa đơn chưa upload thành công hay không.
    pub fn has_pending_uploads(&self, tax_code: &str) -> Result<bool> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM sync_invoices
                    WHERE tax_code = ?1 AND upload_status != 'completed'
                 )",
                params![tax_code],
                |row| row.get(0),
            )
            .map_err(db_error)?;
        Ok(exists)
    }

    /// Xóa dấu vết quét trong ngày sau khi đã xử lý xong lượt quét hôm nay.
    pub fn clear_today_invoices(
        &self,
        tax_code: &str,
        invoice_type: u8,
        sync_date: &str,
    ) -> Result<usize> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        conn.execute(
            "DELETE FROM sync_invoices_today
             WHERE tax_code = ?1 AND invoice_type = ?2 AND invoice_date = ?3",
            params![tax_code, invoice_type, sync_date],
        )
        .map_err(db_error)
    }

    /// Kiểm tra hóa đơn đã xuất hiện trong danh sách quét hôm nay chưa.
    pub fn invoice_seen_today(
        &self,
        tax_code: &str,
        invoice_type: u8,
        invoice_key: &str,
    ) -> Result<bool> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM sync_invoices_today
                    WHERE tax_code = ?1 AND invoice_type = ?2 AND invoice_key = ?3
                 )",
                params![tax_code, invoice_type, invoice_key],
                |row| row.get(0),
            )
            .map_err(db_error)?;
        Ok(exists)
    }
}

// -------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------

pub fn db_error(error: rusqlite::Error) -> TaxSyncError {
    TaxSyncError::Database(error.to_string())
}

pub fn validate_date(date: &str) -> Result<()> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| TaxSyncError::InvalidDate(date.to_string()))
}

fn validate_invoice_type(invoice_type: u8) -> Result<()> {
    if (1..=4).contains(&invoice_type) {
        Ok(())
    } else {
        Err(TaxSyncError::Database(format!(
            "Loại hóa đơn không hợp lệ: {invoice_type}"
        )))
    }
}

/// Chuyển tdlap về ngày lịch Việt Nam (UTC+7).
pub fn invoice_date_vietnam(invoice: &Value) -> Result<String> {
    let value = invoice
        .get("tdlap")
        .and_then(Value::as_str)
        .ok_or(TaxSyncError::MissingField("tdlap"))?;

    let datetime = DateTime::parse_from_rfc3339(value)
        .map_err(|_| TaxSyncError::InvalidDate(value.to_string()))?;

    let vietnam_offset = FixedOffset::east_opt(7 * 60 * 60)
        .ok_or_else(|| TaxSyncError::InvalidDate(value.to_string()))?;

    Ok(datetime
        .with_timezone(&vietnam_offset)
        .format("%Y-%m-%d")
        .to_string())
}

/// Ưu tiên ID hóa đơn do cơ quan thuế cấp.
/// Nếu thiếu ID, sử dụng khóa ghép để dự phòng.
pub fn invoice_key(invoice: &Value) -> Result<String> {
    if let Some(id) = invoice
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.trim().is_empty())
    {
        return Ok(format!("id:{id}"));
    }

    let tax_code = invoice
        .get("nbmst")
        .and_then(Value::as_str)
        .ok_or(TaxSyncError::MissingField("nbmst"))?;

    let form_code = invoice
        .get("khmshdon")
        .map(value_to_string)
        .filter(|s| !s.is_empty())
        .ok_or(TaxSyncError::MissingField("khmshdon"))?;

    let serial = invoice
        .get("khhdon")
        .and_then(Value::as_str)
        .ok_or(TaxSyncError::MissingField("khhdon"))?;

    let number = invoice
        .get("shdon")
        .map(value_to_string)
        .filter(|s| !s.is_empty())
        .ok_or(TaxSyncError::MissingField("shdon"))?;

    Ok(format!("fallback:{tax_code}|{form_code}|{serial}|{number}"))
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

// -------------------------------------------------------------------------
// Các truy vấn phục vụ UI: thống kê, lịch sử, danh sách và reset trạng thái.
// Những hàm này cũng thuộc InvoiceDatabase vì chúng chỉ đọc/ghi bảng hóa đơn.
// -------------------------------------------------------------------------

impl InvoiceDatabase {
    pub fn get_invoice_stats(&self, tax_code: &str) -> Result<InvoiceStats> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;

        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sync_invoices WHERE tax_code = ?1",
                params![tax_code],
                |row| row.get(0),
            )
            .map_err(db_error)?;

        let mut stmt = conn
            .prepare(
                "SELECT invoice_type, detail_status, upload_status, COUNT(*) AS total
                 FROM sync_invoices
                 WHERE tax_code = ?1
                 GROUP BY invoice_type, detail_status, upload_status
                 ORDER BY invoice_type, detail_status, upload_status",
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

    pub fn get_sync_days(&self, tax_code: &str, is_success: Option<bool>) -> Result<Vec<SyncDay>> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;

        let mut stmt = conn
            .prepare(
                "SELECT
                    ROW_NUMBER() OVER (ORDER BY sync_date DESC, invoice_type ASC) AS id,
                    tax_code, invoice_type, sync_date, list_status, status, last_error,
                    attempt_count, started_at, completed_at, updated_at
                 FROM sync_days
                 WHERE tax_code = ?1
                   AND sync_date <> date('now', 'localtime')
                   AND (
                       ?2 IS NULL
                       OR (?2 = 1 AND list_status = 'completed')
                       OR (?2 = 0 AND list_status <> 'completed')
                   )
                 ORDER BY sync_date DESC, invoice_type ASC",
            )
            .map_err(db_error)?;

        let rows = stmt
            .query_map(params![tax_code, is_success], |row| {
                Ok(SyncDay {
                    id: row.get(0)?,
                    tax_code: row.get(1)?,
                    invoice_type: row.get(2)?,
                    sync_date: row.get(3)?,
                    list_status: row.get(4)?,
                    status: row.get(5)?,
                    last_error: row.get(6)?,
                    attempt_count: row.get(7)?,
                    started_at: row.get(8)?,
                    completed_at: row.get(9)?,
                    updated_at: row.get(10)?,
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
        let tax_code = self.tax_code.as_str();

        if let Some(filter_tax_code) = filters
            .tax_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            self.check_tax_code(filter_tax_code)?;
        }

        let nbmst = filters
            .nbmst
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let khhdon = filters
            .khhdon
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let shdon = filters.shdon;
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
                "SELECT COUNT(*) FROM sync_invoices
                 WHERE tax_code = ?1
                   AND (?2 IS NULL OR invoice_type = ?2)
                   AND (?3 IS NULL OR nbnmmst = ?3)
                   AND (?4 IS NULL OR khhdon = ?4)
                   AND (?5 IS NULL OR shdon = ?5)
                   AND (?6 IS NULL OR invoice_date >= ?6)
                   AND (?7 IS NULL OR invoice_date <= ?7)
                   AND (?8 IS NULL OR detail_status = ?8)
                   AND (?9 IS NULL OR upload_status = ?9)",
                params![
                    tax_code,
                    filters.invoice_type,
                    nbmst,
                    khhdon,
                    shdon.map(|v| v as i64),
                    from_date,
                    to_date,
                    detail_status,
                    upload_status
                ],
                |row| row.get(0),
            )
            .map_err(db_error)?;

        let mut stmt = conn
            .prepare(
                "SELECT
                    tax_code, invoice_type, invoice_key, dvtte, tgia, nbmst,
                    khhdon, shdon, khmshdon, invoice_date, tgtcthue, tgtthue,
                    tgtttbso, tthai, ttxly, detail_status, upload_status,
                    detail_attempt_count, upload_attempt_count, detail_error,
                    upload_error, updated_at, completed_at, nbnmmst, nbnmten
                 FROM sync_invoices
                 WHERE tax_code = ?1
                   AND (?2 IS NULL OR invoice_type = ?2)
                   AND (?3 IS NULL OR nbnmmst = ?3)
                   AND (?4 IS NULL OR khhdon = ?4)
                   AND (?5 IS NULL OR shdon = ?5)
                   AND (?6 IS NULL OR invoice_date >= ?6)
                   AND (?7 IS NULL OR invoice_date <= ?7)
                   AND (?8 IS NULL OR detail_status = ?8)
                   AND (?9 IS NULL OR upload_status = ?9)
                 ORDER BY invoice_date DESC, khhdon DESC, shdon DESC
                 LIMIT ?10 OFFSET ?11",
            )
            .map_err(db_error)?;

        let rows = stmt
            .query_map(
                params![
                    tax_code,
                    filters.invoice_type,
                    nbmst,
                    khhdon,
                    shdon.map(|v| v as i64),
                    from_date,
                    to_date,
                    detail_status,
                    upload_status,
                    page_size,
                    offset
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
                        nbnmmst: row.get(23)?,
                        nbnmten: row.get(24)?,
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

    pub fn get_invoices_pending_upload(
        &self,
        tax_code: &str,
        limit: u32,
    ) -> Result<Vec<InvoiceUpload>> {
        self.check_tax_code(tax_code)?;

        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                "SELECT tax_code, invoice_key, invoice_type, summary_json, xml_data
             FROM sync_invoices
             WHERE tax_code = ?1
               AND detail_status = 'completed'
               AND upload_status <> 'completed'
             ORDER BY invoice_type, invoice_key
             LIMIT ?2",
            )
            .map_err(db_error)?;

        let rows = stmt
            .query_map(params![tax_code, limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u8>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
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
                xml_data,
            });
        }

        Ok(invoices)
    }

    /// tax_sync=true: xóa dữ liệu hóa đơn trong khoảng ngày để đồng bộ lại.
    /// app_sync=true: giữ dữ liệu hóa đơn và đưa trạng thái upload về pending.
    pub fn reset_sync_status(
        &self,
        tax_code: &str,
        from_date: &str,
        to_date: &str,
        tax_sync: bool,
        app_sync: bool,
    ) -> Result<()> {
        self.check_tax_code(tax_code)?;
        if !tax_sync && !app_sync {
            return Ok(());
        }
        validate_date(from_date)?;
        validate_date(to_date)?;
        if from_date > to_date {
            return Err(TaxSyncError::Database(
                "Từ ngày không được lớn hơn đến ngày".to_string(),
            ));
        }

        let mut conn = self.connect()?;
        let tx = conn.transaction().map_err(db_error)?;

        if tax_sync {
            tx.execute(
                "UPDATE sync_days SET list_status='pending'
                 WHERE tax_code = ?1 AND sync_date >= ?2 AND sync_date <= ?3",
                params![tax_code, from_date, to_date],
            )
            .map_err(db_error)?;
        }

        if app_sync {
            tx.execute(
                "UPDATE sync_invoices
                 SET upload_status = 'pending',
                     upload_error = NULL,
                     updated_at = CURRENT_TIMESTAMP
                 WHERE tax_code = ?1 AND invoice_date >= ?2 AND invoice_date <= ?3",
                params![tax_code, from_date, to_date],
            )
            .map_err(db_error)?;
        }

        tx.commit().map_err(db_error)?;
        Ok(())
    }

    pub fn insert_invoice_today_if_absent(
        &self,
        tax_code: &str,
        invoice_type: u8,
        invoice_key: &str,
        invoice_date: &str,
    ) -> Result<bool> {
        self.check_tax_code(tax_code)?;
        validate_invoice_type(invoice_type)?;
        validate_date(invoice_date)?;
        let conn = self.connect()?;
        let affected = conn
            .execute(
                "INSERT OR IGNORE INTO sync_invoices_today
                 (tax_code, invoice_type, invoice_key, invoice_date)
                 VALUES (?1, ?2, ?3, ?4)",
                params![tax_code, invoice_type, invoice_key, invoice_date],
            )
            .map_err(db_error)?;
        Ok(affected > 0)
    }

    pub fn invoice_today_exists(
        &self,
        tax_code: &str,
        invoice_type: u8,
        invoice_key: &str,
    ) -> Result<bool> {
        self.check_tax_code(tax_code)?;
        let conn = self.connect()?;
        conn.query_row(
            "SELECT EXISTS (
                SELECT 1 FROM sync_invoices_today
                WHERE tax_code = ?1 AND invoice_type = ?2 AND invoice_key = ?3
             )",
            params![tax_code, invoice_type, invoice_key],
            |row| row.get(0),
        )
        .map_err(db_error)
    }

    pub fn delete_invoices_today_by_date(
        &self,
        tax_code: &str,
        invoice_type: u8,
        invoice_date: &str,
    ) -> Result<usize> {
        self.check_tax_code(tax_code)?;
        validate_invoice_type(invoice_type)?;
        validate_date(invoice_date)?;
        let conn = self.connect()?;
        conn.execute(
            "DELETE FROM sync_invoices_today
             WHERE tax_code = ?1 AND invoice_type = ?2 AND invoice_date = ?3",
            params![tax_code, invoice_type, invoice_date],
        )
        .map_err(db_error)
    }
}

fn validate_tax_code(tax_code: &str) -> Result<String> {
    let tax_code = tax_code.trim();

    if tax_code.is_empty() {
        return Err(TaxSyncError::Database(
            "Mã số thuế không được để trống".to_string(),
        ));
    }

    if !tax_code.bytes().all(|b| b.is_ascii_digit() || b == b'-') {
        return Err(TaxSyncError::Database(
            "Mã số thuế chỉ được chứa chữ số và dấu gạch ngang (-)".to_string(),
        ));
    }

    Ok(tax_code.to_string())
}
