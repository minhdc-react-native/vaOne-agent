use rusqlite::{params, OptionalExtension};

use crate::secret_store::SecretStore;
use crate::{
    database::{db_error, validate_date, Database},
    error::{Result, TaxSyncError},
    models::{TaxAccount, TaxAccountInput},
};

use uuid::Uuid;
// ---------------------------------------------------------
// Quản lý tài khoản mã số thuế dành cho UI
// ---------------------------------------------------------

impl Database {
    /// Lấy tất cả tài khoản, bao gồm cả tài khoản đã tắt.
    pub fn get_tax_accounts(&self) -> Result<Vec<TaxAccount>> {
        let conn = self.connect()?;

        let mut stmt = conn
            .prepare(
                "
                SELECT
                    tax_code,
                    username,
                    password_key,
                    start_date,
                    enabled,
                    invoice_type,
                    auto_sync,
                    access_code,
                    type_app,
                    user_app,
                    pass_app_key,
                    org_unit,
                    auth_status,
                    token,
                    token_expires_at,

                    app_token,
                    app_refresh_token,
                    app_token_expires_at,

                    last_login_at,
                    last_auth_error,
                    last_sync_at,
                    last_sync_status,
                    last_sync_error,
                    created_at,
                    updated_at,
                    org_unit_id,
                    tenant_id,
                    app_token_type,
                    note,
                    url_api,
                    auth_blocked,
                    app_auth_status,
                    app_auth_blocked,
                    last_app_login_at,
                    last_app_auth_error
                FROM tax_accounts
                ORDER BY tax_code
                ",
            )
            .map_err(|e| TaxSyncError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(TaxAccount {
                    tax_code: row.get(0)?,
                    username: row.get(1)?,
                    password_key: row.get(2)?,
                    start_date: row.get(3)?,
                    enabled: row.get::<_, i64>(4)? != 0,
                    invoice_type: row.get(5)?,
                    auto_sync: row.get::<_, i64>(6)? != 0,
                    access_code: row.get(7)?,
                    type_app: row.get(8)?,
                    user_app: row.get(9)?,
                    pass_app_key: row.get(10)?,
                    org_unit: row.get(11)?,
                    auth_status: row.get(12)?,
                    token: row.get(13)?,
                    token_expires_at: row.get(14)?,
                    app_token: row.get(15)?,
                    app_refresh_token: row.get(16)?,
                    app_token_expires_at: row.get(17)?,
                    last_login_at: row.get(18)?,
                    last_auth_error: row.get(19)?,
                    last_sync_at: row.get(20)?,
                    last_sync_status: row.get(21)?,
                    last_sync_error: row.get(22)?,
                    created_at: row.get(23)?,
                    updated_at: row.get(24)?,
                    org_unit_id: row.get(25)?,
                    tenant_id: row.get(26)?,
                    app_token_type: row.get(27)?,
                    note: row.get(28)?,
                    url_api: row.get(29)?,
                    auth_blocked: row.get(30)?,
                    app_auth_status: row.get(31)?,
                    app_auth_blocked: row.get(32)?,
                    last_app_login_at: row.get(33)?,
                    last_app_auth_error: row.get(34)?,
                })
            })
            .map_err(|e| TaxSyncError::Database(e.to_string()))?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| TaxSyncError::Database(e.to_string()))
    }

    /// Thêm tài khoản mới.
    pub fn create_tax_account(&self, input: &TaxAccountInput) -> Result<TaxAccount> {
        validate_tax_account_input(input, true)?;

        let password = input.password_key.trim();

        let password_key = format!("tax:{}:password:{}", input.tax_code.trim(), Uuid::new_v4());
        let store = SecretStore::new();

        store.set_secret(&password_key, password).map_err(|_| {
            TaxSyncError::Database("Không thể lưu mật khẩu thuế vào kho bảo mật".to_string())
        })?;

        let app_password = input.pass_app_key.as_deref().unwrap_or("").trim();

        let pass_app_key = if app_password.is_empty() {
            None
        } else {
            let key = format!(
                "tax:{}:app-password:{}",
                input.tax_code.trim(),
                Uuid::new_v4()
            );

            store.set_secret(&key, app_password).map_err(|_| {
                TaxSyncError::Database(
                    "Không thể lưu mật khẩu ứng dụng vào kho bảo mật".to_string(),
                )
            })?;

            Some(key)
        };

        let conn = self.connect()?;

        conn.execute(
            "
            INSERT INTO tax_accounts (
                tax_code,
                username,
                password_key,
                start_date,
                enabled,
                invoice_type,
                auto_sync,
                access_code,
                type_app,
                user_app,
                pass_app_key,
                org_unit,
                note,
                url_api
            )
            VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14
            )
            ",
            params![
                input.tax_code.trim(),
                input.username.trim(),
                password_key,
                input.start_date,
                input.enabled as i32,
                input.invoice_type,
                input.auto_sync as i32,
                input.access_code.as_deref(),
                input.type_app,
                input.user_app.as_deref(),
                pass_app_key,
                input.org_unit.as_deref(),
                input.note,
                input.url_api
            ],
        )
        .map_err(db_error)?;

        self.get_tax_account(input.tax_code.trim())
    }

    /// Cập nhật cấu hình tài khoản theo mã số thuế.
    ///
    /// Không cập nhật token và trạng thái đồng bộ để tránh
    /// ghi đè trạng thái đang được worker sử dụng.
    pub fn update_tax_account(
        &self,
        tax_code: &str,
        input: &TaxAccountInput,
    ) -> Result<TaxAccount> {
        validate_tax_account_input(input, false)?;

        let tax_code = tax_code.trim();

        if tax_code != input.tax_code.trim() {
            return Err(TaxSyncError::Database(
                "Không được thay đổi mã số thuế khi cập nhật tài khoản".to_string(),
            ));
        }

        let conn = self.connect()?;

        // 1. Lấy các key hiện tại từ SQLite.
        let (old_password_key, old_app_key) = conn
            .query_row(
                "SELECT password_key, pass_app_key
             FROM tax_accounts
             WHERE tax_code = ?1",
                params![tax_code],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)),
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| {
                TaxSyncError::Database(format!("Không tìm thấy tài khoản mã số thuế: {tax_code}"))
            })?;

        let store = SecretStore::new();

        // 2. Nếu nhập mật khẩu thuế mới, lưu vào Keychain bằng key mới.
        // Nếu để trống, giữ nguyên key hiện tại.
        let new_password = input.password_key.trim();

        let next_password_key = if !new_password.is_empty() {
            let key = format!("tax:{tax_code}:password:{}", Uuid::new_v4());

            store.set_secret(&key, new_password).map_err(|_| {
                TaxSyncError::Database("Không thể lưu mật khẩu thuế vào kho bảo mật".to_string())
            })?;

            Some(key)
        } else {
            None
        };

        // 3. Xử lý mật khẩu ứng dụng theo cùng nguyên tắc.
        let new_app_password = input.pass_app_key.as_deref().unwrap_or("").trim();

        let next_app_key = if !new_app_password.is_empty() {
            let key = format!("tax:{tax_code}:app-password:{}", Uuid::new_v4());

            if store.set_secret(&key, new_app_password).is_err() {
                if let Some(key) = &next_password_key {
                    let _ = store.delete_secret(key);
                }

                return Err(TaxSyncError::Database(
                    "Không thể lưu mật khẩu ứng dụng vào kho bảo mật".to_string(),
                ));
            }

            Some(key)
        } else {
            None
        };

        // 4. Chọn key mới nếu đổi mật khẩu, nếu không giữ key cũ.
        let password_key = next_password_key.as_deref().unwrap_or(&old_password_key);

        let pass_app_key = next_app_key.as_deref().or(old_app_key.as_deref());

        // 5. Chỉ lưu key tham chiếu vào SQLite.
        let update_result = conn.execute(
            "
        UPDATE tax_accounts
        SET
            username = ?2,
            password_key = ?3,
            start_date = ?4,
            enabled = ?5,
            invoice_type = ?6,
            auto_sync = ?7,
            access_code = ?8,
            type_app = ?9,
            user_app = ?10,
            pass_app_key = ?11,
            org_unit = ?12,
            note = ?13,
            url_api = ?14,
            
            -- Reset xác thực cơ quan thuế
            auth_status = 'pending',
            token = NULL,
            refresh_token = NULL,
            token_type = NULL,
            token_expires_at = NULL,
            last_login_at = NULL,
            last_auth_error = NULL,
            auth_fail_count = 0,
            auth_blocked = 0,

            -- Reset xác thực ứng dụng
            app_auth_status = 'pending',
            app_token = NULL,
            app_refresh_token = NULL,
            app_token_type = NULL,
            app_token_expires_at = NULL,
            last_app_login_at = NULL,
            last_app_auth_error = NULL,
            app_auth_fail_count = 0,
            app_auth_blocked = 0,

            -- Reset trạng thái đồng bộ gần nhất
            last_sync_at = NULL,
            last_sync_status = 'pending',
            last_sync_error = NULL,

            updated_at = CURRENT_TIMESTAMP
        WHERE tax_code = ?1
        ",
            params![
                tax_code,
                input.username.trim(),
                password_key,
                input.start_date,
                input.enabled as i32,
                input.invoice_type,
                input.auto_sync as i32,
                input.access_code.as_deref(),
                input.type_app,
                input.user_app.as_deref(),
                pass_app_key,
                input.org_unit.as_deref(),
                input.note,
                input.url_api
            ],
        );

        match update_result {
            Ok(changed) if changed > 0 => {
                // 6. Chỉ xóa secret cũ sau khi SQLite đã cập nhật thành công.
                //
                // Có thể chuyển bước dọn dẹp này sang worker riêng để tránh
                // xóa key cũ khi một tác vụ đồng bộ vẫn đang sử dụng nó.
                if next_password_key.is_some() {
                    let _ = store.delete_secret(&old_password_key);
                }

                if next_app_key.is_some() {
                    if let Some(old_key) = old_app_key {
                        let _ = store.delete_secret(&old_key);
                    }
                }

                self.get_tax_account(&input.tax_code)
            }

            Ok(_) => {
                if let Some(key) = &next_password_key {
                    let _ = store.delete_secret(key);
                }

                if let Some(key) = &next_app_key {
                    let _ = store.delete_secret(key);
                }

                Err(TaxSyncError::Database(format!(
                    "Không tìm thấy tài khoản mã số thuế: {tax_code}"
                )))
            }

            Err(error) => {
                // SQLite cập nhật thất bại: dọn các secret mới vừa tạo.
                if let Some(key) = &next_password_key {
                    let _ = store.delete_secret(key);
                }

                if let Some(key) = &next_app_key {
                    let _ = store.delete_secret(key);
                }

                Err(db_error(error))
            }
        }
    }

    /// Xóa tài khoản theo mã số thuế.
    pub fn delete_tax_account(&self, tax_code: &str) -> Result<()> {
        let tax_code = tax_code.trim();

        if tax_code.is_empty() {
            return Err(TaxSyncError::Database(
                "Mã số thuế không được để trống".to_string(),
            ));
        }

        let mut conn = self.connect()?;

        let tx = conn.transaction().map_err(db_error)?;

        // 1. Xóa dữ liệu đồng bộ hóa đơn
        tx.execute(
            "DELETE FROM sync_invoices WHERE tax_code = ?1",
            params![tax_code],
        )
        .map_err(db_error)?;

        // 2. Xóa trạng thái đồng bộ theo ngày
        tx.execute(
            "DELETE FROM sync_days WHERE tax_code = ?1",
            params![tax_code],
        )
        .map_err(db_error)?;

        // 3. Xóa tài khoản thuế
        let changed = tx
            .execute(
                "DELETE FROM tax_accounts WHERE tax_code = ?1",
                params![tax_code],
            )
            .map_err(db_error)?;

        if changed == 0 {
            return Err(TaxSyncError::Database(format!(
                "Không tìm thấy tài khoản mã số thuế: {tax_code}"
            )));
        }

        // Xác nhận toàn bộ thao tác xóa
        tx.commit().map_err(db_error)?;

        Ok(())
    }

    pub fn get_tax_account(&self, tax_code: &str) -> Result<TaxAccount> {
        let conn = self.connect()?;

        let account = conn
            .query_row(
                "
            SELECT
                tax_code,           -- 0
                username,           -- 1
                password_key,       -- 2
                start_date,         -- 3
                enabled,             -- 4
                invoice_type,        -- 5
                auto_sync,           -- 6
                access_code,         -- 7
                type_app,            -- 8
                user_app,            -- 9
                pass_app_key,        -- 10
                org_unit,            -- 11
                auth_status,         -- 12
                token,               -- 13
                token_expires_at,    -- 14
                app_token,           -- 15
                app_refresh_token,   -- 16
                app_token_expires_at,    -- 17
                last_login_at,       -- 18
                last_auth_error,     -- 19
                last_sync_at,        -- 20
                last_sync_status,    -- 21
                last_sync_error,     -- 22
                created_at,          -- 23
                updated_at,          -- 24
                org_unit_id,          -- 25
                tenant_id,           -- 26
                app_token_type,           -- 27
                note,
                url_api,
                auth_blocked,
                app_auth_status,
                app_auth_blocked,
                last_app_login_at,
                last_app_auth_error
            FROM tax_accounts
            WHERE tax_code = ?1
            ",
                params![tax_code.trim()],
                |row| {
                    Ok(TaxAccount {
                        tax_code: row.get(0)?,
                        username: row.get(1)?,
                        password_key: row.get(2)?,
                        start_date: row.get(3)?,
                        enabled: row.get::<_, i64>(4)? != 0,
                        invoice_type: row.get(5)?,
                        auto_sync: row.get::<_, i64>(6)? != 0,
                        access_code: row.get(7)?,
                        type_app: row.get(8)?,
                        user_app: row.get(9)?,
                        pass_app_key: row.get(10)?,
                        org_unit: row.get(11)?,
                        auth_status: row.get(12)?,
                        token: row.get(13)?,
                        token_expires_at: row.get(14)?,

                        app_token: row.get(15)?,
                        app_refresh_token: row.get(16)?,
                        app_token_expires_at: row.get(17)?,

                        last_login_at: row.get(18)?,
                        last_auth_error: row.get(19)?,
                        last_sync_at: row.get(20)?,
                        last_sync_status: row.get(21)?,
                        last_sync_error: row.get(22)?,
                        created_at: row.get(23)?,
                        updated_at: row.get(24)?,
                        org_unit_id: row.get(25)?,
                        tenant_id: row.get(26)?,
                        app_token_type: row.get(27)?,
                        note: row.get(28)?,
                        url_api: row.get(29)?,
                        auth_blocked: row.get(30)?,
                        app_auth_status: row.get(31)?,
                        app_auth_blocked: row.get(32)?,
                        last_app_login_at: row.get(33)?,
                        last_app_auth_error: row.get(34)?,
                    })
                },
            )
            .optional()
            .map_err(db_error)?;

        account.ok_or_else(|| {
            TaxSyncError::Database(format!(
                "Không tìm thấy tài khoản mã số thuế: {}",
                tax_code.trim()
            ))
        })
    }

    /// Kiểm tra mã số thuế đã tồn tại hay chưa.
    pub fn tax_account_exists(&self, tax_code: &str) -> Result<bool> {
        let conn = self.connect()?;

        conn.query_row(
            "SELECT 1 FROM tax_accounts WHERE tax_code = ?1",
            params![tax_code.trim()],
            |_| Ok(()),
        )
        .optional()
        .map(|result| result.is_some())
        .map_err(db_error)
    }

    pub fn get_enabled_tax_accounts(&self) -> Result<Vec<TaxAccount>> {
        let conn = self.connect()?;

        let mut stmt = conn
            .prepare(
                "
            SELECT
                tax_code,
                username,
                password_key,
                start_date,
                enabled,
                invoice_type,
                auto_sync,
                access_code,
                type_app,
                user_app,
                pass_app_key,
                org_unit,
                auth_status,
                token,
                token_expires_at,
                app_token,
                app_refresh_token,
                app_token_expires_at,
                last_login_at,
                last_auth_error,
                last_sync_at,
                last_sync_status,
                last_sync_error,
                created_at,
                updated_at,
                org_unit_id,
                tenant_id,
                app_token_type,
                note,
                url_api,
                auth_blocked,
                app_auth_status,
                app_auth_blocked,
                last_app_login_at,
                last_app_auth_error
            FROM tax_accounts
            WHERE enabled = 1
                AND auth_blocked = 0 
            ORDER BY tax_code
            ",
            )
            .map_err(db_error)?;

        let rows = stmt
            .query_map([], |row| {
                Ok(TaxAccount {
                    tax_code: row.get(0)?,
                    username: row.get(1)?,
                    password_key: row.get(2)?,
                    start_date: row.get(3)?,
                    enabled: row.get::<_, i64>(4)? != 0,
                    invoice_type: row.get(5)?,
                    auto_sync: row.get::<_, i64>(6)? != 0,
                    access_code: row.get(7)?,
                    type_app: row.get(8)?,
                    user_app: row.get(9)?,
                    pass_app_key: row.get(10)?,
                    org_unit: row.get(11)?,
                    auth_status: row.get(12)?,
                    token: row.get(13)?,
                    token_expires_at: row.get(14)?,
                    app_token: row.get(15)?,
                    app_refresh_token: row.get(16)?,
                    app_token_expires_at: row.get(17)?,
                    last_login_at: row.get(18)?,
                    last_auth_error: row.get(19)?,
                    last_sync_at: row.get(20)?,
                    last_sync_status: row.get(21)?,
                    last_sync_error: row.get(22)?,
                    created_at: row.get(23)?,
                    updated_at: row.get(24)?,
                    org_unit_id: row.get(25)?,
                    tenant_id: row.get(26)?,
                    app_token_type: row.get(27)?,
                    note: row.get(28)?,
                    url_api: row.get(29)?,
                    auth_blocked: row.get(30)?,
                    app_auth_status: row.get(31)?,
                    app_auth_blocked: row.get(32)?,
                    last_app_login_at: row.get(33)?,
                    last_app_auth_error: row.get(34)?,
                })
            })
            .map_err(db_error)?;

        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| TaxSyncError::Database(e.to_string()))
    }

    pub fn upsert_tax_account(
        &self,
        tax_code: &str,
        start_date: &str,
        enabled: bool,
    ) -> Result<()> {
        validate_date(start_date)?;

        let conn = self.connect()?;

        conn.execute(
            "
        INSERT INTO tax_accounts (tax_code, start_date, enabled)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(tax_code) DO UPDATE SET
            start_date = excluded.start_date,
            enabled = excluded.enabled,
            updated_at = CURRENT_TIMESTAMP
        ",
            params![tax_code, start_date, enabled as i32],
        )
        .map_err(db_error)?;

        Ok(())
    }
}

fn validate_tax_account_input(input: &TaxAccountInput, is_create: bool) -> Result<()> {
    if input.tax_code.trim().is_empty() {
        return Err(TaxSyncError::Database(
            "Mã số thuế không được để trống".to_string(),
        ));
    }

    if input.username.trim().is_empty() {
        return Err(TaxSyncError::Database(
            "Tên đăng nhập không được để trống".to_string(),
        ));
    }

    // Khi thêm mới, bắt buộc nhập mật khẩu thuế.
    if is_create && input.password_key.trim().is_empty() {
        return Err(TaxSyncError::Database(
            "Mật khẩu thuế không được để trống".to_string(),
        ));
    }

    if input.type_app != "vaOnline" && input.type_app != "vaOne" {
        return Err(TaxSyncError::Database(
            "type_app chỉ chấp nhận vaOnline hoặc vaOne".to_string(),
        ));
    }

    // Khi bật đồng bộ tự động, bắt buộc có đủ thông tin ứng dụng.
    if input.auto_sync {
        if input.access_code.as_deref().unwrap_or("").trim().is_empty() {
            return Err(TaxSyncError::Database(
                "Mã truy cập không được để trống khi bật đồng bộ tự động".to_string(),
            ));
        }

        if input.org_unit.as_deref().unwrap_or("").trim().is_empty() {
            return Err(TaxSyncError::Database(
                "Đơn vị tổ chức không được để trống khi bật đồng bộ tự động".to_string(),
            ));
        }

        if input.user_app.as_deref().unwrap_or("").trim().is_empty() {
            return Err(TaxSyncError::Database(
                "Tên đăng nhập ứng dụng không được để trống khi bật đồng bộ tự động".to_string(),
            ));
        }

        // Chỉ bắt buộc mật khẩu ứng dụng khi thêm mới.
        if is_create
            && input
                .pass_app_key
                .as_deref()
                .unwrap_or("")
                .trim()
                .is_empty()
        {
            return Err(TaxSyncError::Database(
                "Mật khẩu ứng dụng không được để trống khi thêm mới và bật đồng bộ tự động"
                    .to_string(),
            ));
        }
    }

    validate_date(&input.start_date)?;

    Ok(())
}
