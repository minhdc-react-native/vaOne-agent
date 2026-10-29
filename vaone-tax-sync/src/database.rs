use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};

use crate::error::{Result, TaxSyncError};

#[derive(Clone)]
pub struct Database {
    path: PathBuf,
}

impl Database {
    /// Khởi tạo database điều phối.
    ///
    /// Database này chỉ quản lý tài khoản và cấu hình chung.
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| TaxSyncError::Database(e.to_string()))?;
        }

        let db = Self { path };
        db.initialize()?;

        Ok(db)
    }

    /// Mở kết nối database điều phối.
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

    /// Chỉ khởi tạo các bảng thuộc database điều phối.
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

                -- Cấu hình đồng bộ tự động lên ứng dụng ERP.
                auto_sync INTEGER NOT NULL DEFAULT 0
                    CHECK (auto_sync IN (0, 1)),

                access_code TEXT,

                type_app TEXT NOT NULL DEFAULT 'vaOnline'
                    CHECK (
                        type_app IN ('vaOnline', 'vaOne', 'vaShop')
                    ),

                user_app TEXT,
                pass_app_key TEXT,
                org_unit TEXT,

                -- Trạng thái xác thực cơ quan thuế.
                auth_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (
                        auth_status IN (
                            'pending', 'authenticated', 'failed'
                        )
                    ),

                token TEXT,
                refresh_token TEXT,
                token_type TEXT,
                token_expires_at TEXT,

                -- Trạng thái xác thực ứng dụng ERP.
                app_token TEXT,
                app_refresh_token TEXT,
                app_token_type TEXT,
                app_token_expires_at TEXT,

                last_login_at TEXT,
                last_auth_error TEXT,

                app_auth_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (
                        app_auth_status IN (
                            'pending', 'authenticated', 'failed'
                        )
                    ),

                app_auth_fail_count INTEGER NOT NULL DEFAULT 0
                    CHECK (app_auth_fail_count >= 0),

                app_auth_blocked INTEGER NOT NULL DEFAULT 0
                    CHECK (app_auth_blocked IN (0, 1)),

                last_app_login_at TEXT,
                last_app_auth_error TEXT,

                -- Trạng thái đồng bộ tổng hợp để UI hiển thị.
                last_sync_at TEXT,
                last_sync_status TEXT NOT NULL DEFAULT 'pending'
                    CHECK (
                        last_sync_status IN (
                            'pending', 'running', 'completed', 'failed'
                        )
                    ),

                last_sync_error TEXT,

                -- Thời gian tạo và cập nhật.
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

                org_unit_id TEXT,
                tenant_id TEXT,

                note TEXT,
                url_api TEXT,

                -- Trạng thái xác thực cơ quan thuế.
                auth_fail_count INTEGER NOT NULL DEFAULT 0
                    CHECK (auth_fail_count >= 0),

                auth_blocked INTEGER NOT NULL DEFAULT 0
                    CHECK (auth_blocked IN (0, 1))
            );

            CREATE INDEX IF NOT EXISTS idx_tax_accounts_enabled
                ON tax_accounts(enabled, auth_status);

            CREATE INDEX IF NOT EXISTS idx_tax_accounts_app_auth
                ON tax_accounts(enabled, app_auth_status);
            ",
        )
        .map_err(|e| TaxSyncError::Database(e.to_string()))?;

        Ok(())
    }

    // ---------------------------------------------------------
    // 2. Xác thực cơ quan thuế
    // ---------------------------------------------------------

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
                "
                SELECT auth_blocked
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
    // 3. Xác thực ứng dụng ERP
    // ---------------------------------------------------------

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
            params![
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
}

pub fn db_error(error: rusqlite::Error) -> TaxSyncError {
    TaxSyncError::Database(error.to_string())
}

pub fn validate_date(date: &str) -> Result<()> {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|_| ())
        .map_err(|_| TaxSyncError::InvalidDate(date.to_string()))
}
