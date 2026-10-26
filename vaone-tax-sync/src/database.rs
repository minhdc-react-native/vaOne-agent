use chrono::{DateTime, Duration, FixedOffset, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use crate::{
    error::{Result, TaxSyncError},
    models::InvoiceDetailRetry,
};

const STATUS_PENDING: &str = "pending";
const STATUS_RUNNING: &str = "running";
const STATUS_COMPLETED: &str = "completed";
const STATUS_FAILED: &str = "failed";

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
pub struct Database {
    path: PathBuf,
}

impl Database {
    /// Khởi tạo database và tạo các bảng nếu chưa tồn tại.
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| TaxSyncError::Database(e.to_string()))?;
        }

        let db = Self { path };
        db.initialize()?;

        Ok(db)
    }

    pub fn connect(&self) -> Result<Connection> {
        let conn =
            Connection::open(&self.path).map_err(|e| TaxSyncError::Database(e.to_string()))?;

        conn.busy_timeout(std::time::Duration::from_secs(10))
            .map_err(|e| TaxSyncError::Database(e.to_string()))?;

        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            ",
        )
        .map_err(|e| TaxSyncError::Database(e.to_string()))?;

        Ok(conn)
    }

    fn initialize(&self) -> Result<()> {
        let conn = self.connect()?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS tax_accounts (
                tax_code TEXT PRIMARY KEY,

                -- Thông tin đăng nhập hóa đơn điện tử.
                username TEXT NOT NULL,
                password_key TEXT NOT NULL,

                -- Cấu hình đồng bộ.
                start_date TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1
                    CHECK (enabled IN (0, 1)),

                invoice_type TEXT NOT NULL DEFAULT '[1,2,3,4]',
                
                -- Cấu hình đồng bộ tự động đến ứng dụng ERP.
                auto_sync INTEGER NOT NULL DEFAULT 0
                    CHECK (auto_sync IN (0, 1)),

                access_code TEXT,

                type_app TEXT NOT NULL DEFAULT 'vaOnline'
                    CHECK (type_app IN ('vaOnline', 'vaOne', 'vaShop')),

                user_app TEXT,
                pass_app_key TEXT,
                org_unit TEXT,

                -- Trạng thái xác thực cơ quan thuế.
                auth_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (auth_status IN (
                        'pending', 'authenticated', 'failed'
                    )),
                token TEXT,
                refresh_token TEXT,
                token_type TEXT,
                token_expires_at TEXT,

                app_token TEXT,
                app_refresh_token TEXT,
                app_token_type TEXT,
                app_token_expires_at TEXT,

                last_login_at TEXT,
                last_auth_error TEXT,

                -- Trạng thái đồng bộ gần nhất.
                last_sync_at TEXT,
                last_sync_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (last_sync_status IN (
                        'pending', 'running', 'completed', 'failed'
                    )),
                last_sync_error TEXT,

                -- Thời gian tạo và cập nhật.
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

                org_unit_id TEXT,
                tenant_id TEXT,

                note TEXT,
                url_api TEXT,
                -- Trạng thái xác thực truy cập cơ quan thuế
                auth_fail_count INTEGER NOT NULL DEFAULT 0
                    CHECK (auth_fail_count >= 0),
                auth_blocked INTEGER NOT NULL DEFAULT 0
                    CHECK (auth_blocked IN (0, 1)),         
                -- Trạng thái xác thực truy cập ứng dụng.
                app_auth_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (app_auth_status IN (
                        'pending', 'authenticated', 'failed'
                    )),
                app_auth_fail_count INTEGER NOT NULL DEFAULT 0
                    CHECK (app_auth_fail_count >= 0),
                app_auth_blocked INTEGER NOT NULL DEFAULT 0
                    CHECK (app_auth_blocked IN (0, 1)),
                last_app_login_at TEXT,
                last_app_auth_error TEXT           
            );

            CREATE TABLE IF NOT EXISTS sync_days (
                tax_code TEXT NOT NULL,
                invoice_type INTEGER NOT NULL
                    CHECK (invoice_type IN (1, 2, 3, 4)),

                sync_date TEXT NOT NULL,

                -- Trạng thái tải đầy đủ danh sách hóa đơn của ngày.
                list_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (list_status IN (
                        'pending', 'running', 'completed', 'failed'
                    )),

                -- Trạng thái tổng hợp của ngày.
                status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN (
                        'pending', 'running', 'completed', 'failed'
                    )),

                last_error TEXT,
                attempt_count INTEGER NOT NULL DEFAULT 0,

                started_at TEXT,
                completed_at TEXT,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

                PRIMARY KEY (tax_code, invoice_type, sync_date),

                FOREIGN KEY (tax_code)
                    REFERENCES tax_accounts(tax_code)
                    ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS sync_invoices (
                tax_code TEXT NOT NULL,
                invoice_type INTEGER NOT NULL
                    CHECK (invoice_type IN (1, 2, 3, 4)),
                invoice_key TEXT NOT NULL,

                dvtte TEXT,
                tgia REAL,
                nbmst TEXT,
                nbten TEXT,
                khhdon TEXT,
                shdon INTEGER,
                khmshdon INTEGER,
                invoice_date TEXT NOT NULL,

                tgtcthue REAL,
                tgtthue REAL,
                tgtttbso REAL,
                tthai INTEGER,
                ttxly INTEGER,

                -- JSON nhận được từ API danh sách.
                summary_json TEXT NOT NULL,

                -- JSON đầy đủ sau khi lấy chi tiết.
                xml_data TEXT,

                -- Trạng thái lấy chi tiết.
                detail_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (detail_status IN (
                        'pending', 'running', 'completed', 'failed'
                    )),

                -- Trạng thái tải dữ liệu lên API đích.
                upload_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (upload_status IN (
                        'pending', 'running', 'completed', 'failed'
                    )),

                -- Số lần thử riêng cho từng công đoạn.
                detail_attempt_count INTEGER NOT NULL DEFAULT 0,
                upload_attempt_count INTEGER NOT NULL DEFAULT 0,

                -- Lỗi riêng để retry đúng công đoạn.
                detail_error TEXT,
                upload_error TEXT,

                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                completed_at TEXT,

                PRIMARY KEY (tax_code, invoice_type, invoice_key),

                FOREIGN KEY (
                    tax_code, invoice_type, invoice_date
                )
                REFERENCES sync_days (
                    tax_code, invoice_type, sync_date
                )
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

            -- Tìm các ngày cần xử lý hoặc retry.
            CREATE INDEX IF NOT EXISTS idx_sync_days_status
                ON sync_days(status, sync_date);

            CREATE INDEX IF NOT EXISTS idx_sync_days_account_status
                ON sync_days(tax_code, status, sync_date);

            -- Tìm hóa đơn chưa lấy chi tiết hoặc chưa upload thành công.
            CREATE INDEX IF NOT EXISTS idx_sync_invoices_retry
                ON sync_invoices(
                    tax_code,
                    invoice_type,
                    invoice_date,
                    detail_status,
                    upload_status
                );

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_detail_retry
                ON sync_invoices (
                    tax_code,
                    invoice_type,
                    detail_status,
                    updated_at
                );

            -- Tìm tài khoản được bật để worker tự động xử lý.
            CREATE INDEX IF NOT EXISTS idx_tax_accounts_enabled
                ON tax_accounts(enabled, auth_status);

            CREATE INDEX IF NOT EXISTS idx_sync_days_tax_type_date
                ON sync_days (tax_code, invoice_type, sync_date DESC);

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_tax_date
                ON sync_invoices (tax_code, invoice_date DESC);

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_tax_type_date
                ON sync_invoices (tax_code, invoice_type, invoice_date DESC);

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_tax_detail_status
                ON sync_invoices (tax_code, detail_status);

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_tax_upload_status
                ON sync_invoices (tax_code, upload_status);


            CREATE INDEX IF NOT EXISTS idx_sync_invoices_pending_details
                ON sync_invoices(detail_status)
                WHERE detail_status <> 'completed';

            CREATE INDEX IF NOT EXISTS idx_sync_invoices_pending_upload
                ON sync_invoices(upload_status)
                WHERE upload_status <> 'completed';

            ",
        )
        .map_err(|e| TaxSyncError::Database(e.to_string()))?;

        Ok(())
    }

    // ---------------------------------------------------------
    // 1. Quản lý mã số thuế
    // ---------------------------------------------------------

    pub fn get_start_date(&self, tax_code: &str) -> Result<Option<String>> {
        let conn = self.connect()?;

        conn.query_row(
            "
            SELECT start_date
            FROM tax_accounts
            WHERE tax_code = ?1 AND enabled = 1
            ",
            params![tax_code],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_error)
    }

    pub fn set_auth_success(
        &self,
        tax_code: &str,
        token: &str,
        token_expires_at: &str,
    ) -> Result<()> {
        let conn = self.connect()?;

        conn.execute(
            "
                UPDATE tax_accounts
                SET token = ?2,
                    token_expires_at = ?3,
                    auth_status = 'authenticated',
                    last_login_at = CURRENT_TIMESTAMP,
                    last_auth_error = NULL,
                    auth_fail_count = 0,
                    auth_blocked = 0,
                    updated_at = CURRENT_TIMESTAMP
                WHERE tax_code = ?1
                ",
            params![tax_code, token, token_expires_at],
        )
        .map_err(db_error)?;

        Ok(())
    }

    pub fn set_auth_failure(&self, tax_code: &str, error: &str) -> Result<bool> {
        let conn = self.connect()?;

        conn.execute(
            "
        UPDATE tax_accounts
        SET auth_fail_count = auth_fail_count + 1,
            auth_status = 'failed',
            auth_blocked = CASE
                WHEN auth_fail_count + 1 >= 3 THEN 1
                ELSE 0
            END,
            last_auth_error = ?2,
            last_login_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE tax_code = ?1
          AND auth_blocked = 0
        ",
            params![tax_code, error],
        )
        .map_err(db_error)?;

        let blocked: bool = conn
            .query_row(
                "SELECT auth_blocked FROM tax_accounts WHERE tax_code = ?1",
                params![tax_code],
                |row| row.get(0),
            )
            .map_err(db_error)?;

        Ok(blocked)
    }

    pub fn set_app_auth_success(
        &self,
        tax_code: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        token_type: &str,
        expires_at: &str,
        org_unit_id: Option<&str>,
        tenant_id: Option<&str>,
    ) -> Result<()> {
        let conn = self.connect()?;

        conn.execute(
            "
        UPDATE tax_accounts
        SET app_token = ?2,
            app_refresh_token = ?3,
            app_token_type = ?4,
            app_token_expires_at = ?5,
            org_unit_id = COALESCE(?6, org_unit_id),
            tenant_id = COALESCE(?7, tenant_id),
            app_auth_status = 'authenticated',
            last_app_login_at = CURRENT_TIMESTAMP,
            last_app_auth_error = NULL,
            app_auth_fail_count = 0,
            app_auth_blocked = 0,
            updated_at = CURRENT_TIMESTAMP
        WHERE tax_code = ?1
        ",
            rusqlite::params![
                tax_code,
                access_token,
                refresh_token,
                token_type,
                expires_at,
                org_unit_id,
                tenant_id
            ],
        )
        .map_err(db_error)?;

        Ok(())
    }

    pub fn set_app_auth_failure(&self, tax_code: &str, error: &str) -> Result<bool> {
        let conn = self.connect()?;

        conn.execute(
            "
        UPDATE tax_accounts
        SET app_auth_fail_count = app_auth_fail_count + 1,
            app_auth_status = 'failed',
            app_auth_blocked = CASE
                WHEN app_auth_fail_count + 1 >= 3 THEN 1
                ELSE 0
            END,
            last_app_auth_error = ?2,
            last_app_login_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE tax_code = ?1
          AND app_auth_blocked = 0
        ",
            params![tax_code, error],
        )
        .map_err(db_error)?;

        let blocked: bool = conn
            .query_row(
                "
            SELECT app_auth_blocked
            FROM tax_accounts
            WHERE tax_code = ?1
            ",
                params![tax_code],
                |row| row.get(0),
            )
            .map_err(db_error)?;

        Ok(blocked)
    }

    // ---------------------------------------------------------
    // 2. Quản lý trạng thái từng ngày
    // ---------------------------------------------------------

    pub fn ensure_sync_day(&self, tax_code: &str, invoice_type: u8, sync_date: &str) -> Result<()> {
        validate_date(sync_date)?;

        let conn = self.connect()?;

        conn.execute(
            "
            INSERT INTO sync_days (
                tax_code, invoice_type, sync_date
            )
            VALUES (?1, ?2, ?3)
            ON CONFLICT(tax_code, invoice_type, sync_date)
            DO NOTHING
            ",
            params![tax_code, invoice_type, sync_date],
        )
        .map_err(db_error)?;

        Ok(())
    }

    pub fn set_day_running(&self, tax_code: &str, invoice_type: u8, sync_date: &str) -> Result<()> {
        let conn = self.connect()?;

        conn.execute(
            "
            UPDATE sync_days
            SET
                list_status = 'running',
                status = 'running',
                last_error = NULL,
                attempt_count = attempt_count + 1,
                started_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE tax_code = ?1
            AND invoice_type = ?2
            AND sync_date = ?3;
            ",
            params![tax_code, invoice_type, sync_date],
        )
        .map_err(db_error)?;

        Ok(())
    }

    pub fn set_day_pending(&self, tax_code: &str, invoice_type: u8, sync_date: &str) -> Result<()> {
        let conn = self.connect()?;

        conn.execute(
            "
            UPDATE sync_days
            SET
                list_status = 'pending',
                status = 'pending',
                last_error = NULL,
                attempt_count = attempt_count + 1,
                started_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE tax_code = ?1
            AND invoice_type = ?2
            AND sync_date = ?3;
            ",
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
        let conn = self.connect()?;

        conn.execute(
            r#"
        UPDATE sync_days
        SET
            list_status = 'failed',
            status = 'failed',
            last_error = ?4,
            updated_at = CURRENT_TIMESTAMP
        WHERE tax_code = ?1
          AND invoice_type = ?2
          AND sync_date = ?3
        "#,
            rusqlite::params![tax_code, invoice_type, sync_date, error],
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
        let conn = self.connect()?;

        let list_status = if *is_today { "pending" } else { "completed" };

        conn.execute(
            "
        UPDATE sync_days
        SET list_status = ?1,
            last_error = NULL,
            updated_at = CURRENT_TIMESTAMP
        WHERE tax_code = ?2
          AND invoice_type = ?3
          AND sync_date = ?4
        ",
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
        let conn = self.connect()?;

        conn.execute(
            "
            UPDATE sync_days
            SET status = 'failed',
                last_error = ?4,
                updated_at = CURRENT_TIMESTAMP
            WHERE tax_code = ?1
              AND invoice_type = ?2
              AND sync_date = ?3
            ",
            params![tax_code, invoice_type, sync_date, error],
        )
        .map_err(db_error)?;

        Ok(())
    }

    // ---------------------------------------------------------
    // 3. Lưu hóa đơn danh sách
    // ---------------------------------------------------------

    /// Lưu nguyên JSON danh sách từ cơ quan thuế.
    ///
    /// Hàm này không đặt lại trạng thái chi tiết/tải lên của
    /// hóa đơn đã tồn tại.
    ///
    /// Phải gọi ensure_sync_day trước khi gọi hàm này.
    pub fn save_invoice_summary(
        &self,
        tax_code: &str,
        invoice_type: u8,
        summary: &Value,
        is_today: bool,
    ) -> Result<String> {
        let invoice_date = invoice_date_vietnam(summary)?;
        let invoice_key = invoice_key(summary)?;

        let summary_json = serde_json::to_string(summary)
            .map_err(|e| TaxSyncError::InvalidInvoice(e.to_string()))?;

        let mut conn = self.connect()?;
        let tx = conn.transaction().map_err(db_error)?;

        tx.execute(
            "
        INSERT INTO sync_invoices (
            tax_code, invoice_type, invoice_key, invoice_date,
            dvtte, tgia, nbmst, khhdon, shdon, khmshdon,
            tgtcthue, tgtthue, tgtttbso, tthai, ttxly, nbten,
            summary_json, detail_status, upload_status
        )
        VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,
            ?9, ?10, ?11, ?12, ?13, ?14, ?15,
            ?16, ?17, 'pending', 'pending'
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
            nbten = excluded.nbten,
            summary_json = excluded.summary_json,
            updated_at = CURRENT_TIMESTAMP
        ",
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
                summary["nbten"].as_str(),
                summary_json,
            ],
        )
        .map_err(db_error)?;

        if is_today {
            tx.execute(
                "
            INSERT OR IGNORE INTO sync_invoices_today (
                tax_code,
                invoice_type,
                invoice_key,
                invoice_date
            )
            VALUES (?1, ?2, ?3, ?4)
            ",
                params![tax_code, invoice_type, invoice_key, invoice_date,],
            )
            .map_err(db_error)?;
        }

        tx.commit().map_err(db_error)?;

        Ok(invoice_key)
    }

    // ---------------------------------------------------------
    // 4. Lưu chi tiết hóa đơn
    // ---------------------------------------------------------

    pub fn get_invoices_pending_detail(
        &self,
        tax_code: &str,
        invoice_type: u8,
        limit: u32,
    ) -> Result<Vec<InvoiceDetailRetry>> {
        let conn = self.connect()?;

        let mut stmt = conn
            .prepare(
                "
            SELECT
                invoice_key,
                nbmst,
                khhdon,
                shdon,
                khmshdon
            FROM sync_invoices
            WHERE tax_code = ?1
              AND invoice_type = ?2
              AND detail_status IN ('pending', 'failed')
              AND nbmst IS NOT NULL
              AND khhdon IS NOT NULL
              AND shdon IS NOT NULL
              AND khmshdon IS NOT NULL
            ORDER BY updated_at ASC
            LIMIT ?3
            ",
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

    /// Lưu JSON chi tiết. Chưa có nghĩa là đã tải lên API đích.
    pub fn save_invoice_detail(
        &self,
        tax_code: &str,
        invoice_type: u8,
        invoice_key: &str,
        detail: &Option<Value>,
        xml: &Option<String>,
    ) -> Result<()> {
        let detail_data = detail
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| TaxSyncError::InvalidInvoice(e.to_string()))?;

        let conn = self.connect()?;

        conn.execute(
            "
        UPDATE sync_invoices
        SET summary_json = CASE
                WHEN ?4 IS NOT NULL THEN ?4
                ELSE summary_json
            END,
            xml_data = ?5,
            detail_status = 'completed',
            detail_error = NULL,
            updated_at = CURRENT_TIMESTAMP
        WHERE tax_code = ?1
          AND invoice_type = ?2
          AND invoice_key = ?3
        ",
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
        let conn = self.connect()?;

        conn.execute(
            "
            UPDATE sync_invoices
            SET detail_status = 'failed',
                attempt_count = attempt_count + 1,
                detail_error = ?4,
                updated_at = CURRENT_TIMESTAMP
            WHERE tax_code = ?1
              AND invoice_type = ?2
              AND invoice_key = ?3
            ",
            params![tax_code, invoice_type, invoice_key, error],
        )
        .map_err(db_error)?;

        Ok(())
    }

    // ---------------------------------------------------------
    // 5. Trạng thái tải lên API đích
    // ---------------------------------------------------------

    pub fn set_upload_completed(
        &self,
        tax_code: &str,
        invoice_type: u8,
        invoice_key: &str,
    ) -> Result<()> {
        let conn = self.connect()?;

        conn.execute(
            "
            UPDATE sync_invoices
            SET upload_status = 'completed',
                detail_error = NULL,
                updated_at = CURRENT_TIMESTAMP
            WHERE tax_code = ?1
              AND invoice_type = ?2
              AND invoice_key = ?3
              AND detail_status = 'completed'
              AND xml_data IS NOT NULL
            ",
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
        let conn = self.connect()?;

        conn.execute(
            "
            UPDATE sync_invoices
            SET upload_status = 'failed',
                attempt_count = attempt_count + 1,
                detail_error = ?4,
                updated_at = CURRENT_TIMESTAMP
            WHERE tax_code = ?1
              AND invoice_type = ?2
              AND invoice_key = ?3
            ",
            params![tax_code, invoice_type, invoice_key, error],
        )
        .map_err(db_error)?;

        Ok(())
    }

    // ---------------------------------------------------------
    // 6. Truy vấn hóa đơn cần retry
    // ---------------------------------------------------------

    /// Lấy hóa đơn chưa lấy chi tiết hoặc chưa tải lên thành công.
    pub fn get_pending_invoices(
        &self,
        tax_code: &str,
        invoice_type: u8,
        sync_date: &str,
    ) -> Result<Vec<PendingInvoice>> {
        let conn = self.connect()?;

        let mut stmt = conn
            .prepare(
                "
                SELECT
                    invoice_key,
                    invoice_date,
                    summary_json,
                    xml_data,
                    detail_status,
                    upload_status,
                    attempt_count,
                    detail_error
                FROM sync_invoices
                WHERE tax_code = ?1
                  AND invoice_type = ?2
                  AND invoice_date = ?3
                  AND (
                      detail_status != 'completed'
                      OR upload_status != 'completed'
                  )
                ORDER BY invoice_key
                ",
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

    // ---------------------------------------------------------
    // 7. Xác định ngày đã hoàn tất hay chưa
    // ---------------------------------------------------------

    /// Chỉ hoàn tất ngày khi:
    /// - Danh sách đã được lấy đầy đủ.
    /// - Không còn hóa đơn nào chưa lấy chi tiết hoặc chưa tải lên.
    pub fn refresh_day_status(
        &self,
        tax_code: &str,
        invoice_type: u8,
        sync_date: &str,
    ) -> Result<bool> {
        let conn = self.connect()?;

        let list_status: Option<String> = conn
            .query_row(
                "
                SELECT list_status
                FROM sync_days
                WHERE tax_code = ?1
                  AND invoice_type = ?2
                  AND sync_date = ?3
                ",
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
                "
                SELECT COUNT(*)
                FROM sync_invoices
                WHERE tax_code = ?1
                  AND invoice_type = ?2
                  AND invoice_date = ?3
                  AND (
                      detail_status != 'completed'
                      OR upload_status != 'completed'
                  )
                ",
                params![tax_code, invoice_type, sync_date],
                |row| row.get(0),
            )
            .map_err(db_error)?;

        if pending_count > 0 {
            conn.execute(
                "
                UPDATE sync_days
                SET status = 'failed',
                    last_error = 'Some invoices are still pending or failed',
                    updated_at = CURRENT_TIMESTAMP
                WHERE tax_code = ?1
                  AND invoice_type = ?2
                  AND sync_date = ?3
                ",
                params![tax_code, invoice_type, sync_date],
            )
            .map_err(db_error)?;

            return Ok(false);
        }

        conn.execute(
            "
            UPDATE sync_days
            SET status = 'completed',
                last_error = NULL,
                updated_at = CURRENT_TIMESTAMP
            WHERE tax_code = ?1
              AND invoice_type = ?2
              AND sync_date = ?3
            ",
            params![tax_code, invoice_type, sync_date],
        )
        .map_err(db_error)?;

        Ok(true)
    }

    /// Lấy danh sách ngày chưa hoàn tất để scheduler xử lý lại.
    pub fn get_incomplete_days(&self, tax_code: &str, invoice_type: u8) -> Result<Vec<String>> {
        let conn = self.connect()?;

        let mut stmt = conn
            .prepare(
                "
                SELECT sync_date
                FROM sync_days
                WHERE tax_code = ?1
                  AND invoice_type = ?2
                  AND list_status != 'completed'
                ORDER BY sync_date ASC
                ",
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
}

// -------------------------------------------------------------
// Helpers
// -------------------------------------------------------------

pub fn db_error(error: rusqlite::Error) -> TaxSyncError {
    TaxSyncError::Database(error.to_string())
}

pub fn validate_date(date: &str) -> Result<()> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| TaxSyncError::InvalidDate(date.to_string()))
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
