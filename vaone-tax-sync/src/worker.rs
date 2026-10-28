use crate::auth::VaOnlineOrgUnit;
use crate::models::InvoiceUpload;
use crate::{
    auth::{get_current_year, ApplicationLoginResponse, TaxAuthService, TypeApp},
    client::request_delay,
    database::Database,
    error::TaxSyncError,
    models::{SyncRequest, TaxAccount},
    sync::{SyncObserver, TaxSyncService},
    SecretStore,
};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, NaiveDateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::Notify;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

#[derive(Deserialize)]
struct TimeApiResponse {
    #[serde(rename = "dateTime")]
    date_time: String,
}

const DEFAULT_POLL_INTERVAL_SECS: u64 = 60;
const DEFAULT_DELAY_MS: u64 = 3 * 1000;
const TOKEN_LIFETIME_HOURS: i64 = 24;

fn parse_invoice_types(raw: &str) -> Result<Vec<u8>> {
    let types: Vec<u8> = serde_json::from_str(raw)?;

    if types.iter().any(|t| !(1..=4).contains(t)) {
        return Err(anyhow!("invoice_type chỉ chấp nhận các giá trị 1, 2, 3, 4"));
    }

    let mut unique = HashSet::new();

    Ok(types.into_iter().filter(|t| unique.insert(*t)).collect())
}

pub async fn vietnam_today() -> Result<NaiveDate> {
    let response: TimeApiResponse = reqwest::Client::new()
        .get("https://timeapi.io/api/Time/current/zone")
        .query(&[("timeZone", "Asia/Ho_Chi_Minh")])
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .context("Không lấy được thời gian từ máy chủ")?;

    let today = NaiveDateTime::parse_from_str(&response.date_time, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(&response.date_time, "%Y-%m-%dT%H:%M:%S"))?
        .date();

    Ok(today)
}

async fn vietnam_yesterday() -> Result<NaiveDate> {
    vietnam_today()
        .await?
        .pred_opt()
        .ok_or_else(|| anyhow!("Không xác định được ngày hôm qua"))
}

/// Đọc mật khẩu từ macOS Keychain.
///
/// Ứng dụng Tauri cần triển khai trait này bằng Keychain thực tế.
/// Không lưu mật khẩu dạng plaintext trong SQLite.
pub trait PasswordProvider: Send + Sync {
    fn get_password(&self, password_key: &str) -> Result<String>;
}

/// Giải captcha SVG lấy từ cơ quan thuế.
pub trait CaptchaSolver: Send + Sync {
    fn solve(&self, svg: &str) -> std::result::Result<String, String>;
}

#[derive(Clone)]
pub struct TaxSyncWorkerConfig {
    /// Các loại hóa đơn cần đồng bộ: 1, 2, 3, 4.
    pub invoice_types: Vec<u8>,

    /// Khoảng nghỉ giữa các request tới cơ quan thuế.
    pub delay_ms: u64,

    /// Khoảng nghỉ giữa các lượt quét worker.
    pub poll_interval_secs: u64,
}

impl TaxSyncWorkerConfig {
    pub fn new() -> Self {
        Self {
            invoice_types: vec![1, 2, 3, 4],
            delay_ms: DEFAULT_DELAY_MS,
            poll_interval_secs: DEFAULT_POLL_INTERVAL_SECS,
        }
    }
}

pub struct TaxSyncWorker {
    db: Database,
    auth: TaxAuthService,
    captcha_solver: Arc<dyn CaptchaSolver>,
    observer: Arc<dyn SyncObserver>,
    config: TaxSyncWorkerConfig,
    upload_notify: Arc<Notify>,
    update_notify: Arc<Notify>,
}

impl TaxSyncWorker {
    pub fn new(
        db: Database,
        captcha_solver: Arc<dyn CaptchaSolver>,
        observer: Arc<dyn SyncObserver>,
        config: TaxSyncWorkerConfig,
    ) -> Result<Self> {
        if config.invoice_types.is_empty()
            || config.invoice_types.iter().any(|t| !(1..=4).contains(t))
        {
            return Err(anyhow!(
                "invoice_types chỉ chấp nhận các giá trị 1, 2, 3, 4"
            ));
        }

        let auth = TaxAuthService::new()?;

        Ok(Self {
            db,
            auth,
            captcha_solver,
            observer,
            config,
            upload_notify: Arc::new(Notify::new()),
            update_notify: Arc::new(Notify::new()),
        })
    }

    pub fn notify_upload(&self) {
        self.upload_notify.notify_one();
    }

    pub fn notify_update(&self) {
        self.update_notify.notify_one();
    }

    pub async fn run_workers(&self, cancel: CancellationToken) {
        tokio::join!(
            self.run(cancel.clone()),
            self.run_upload(cancel.clone()),
            self.run_update(cancel.clone()),
        );
    }

    async fn run(&self, cancel: CancellationToken) {
        loop {
            if cancel.is_cancelled() {
                break;
            }
            println!("[tax-sync] RUN_ONCE");

            if let Err(error) = self.run_once(cancel.clone()).await {
                eprintln!("[tax-sync] Worker iteration failed: {error:#}");
            }

            if cancel.is_cancelled() {
                break;
            }

            // self.run_update(cancel.clone()).await;

            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = sleep(Duration::from_secs(
                    self.config.poll_interval_secs.max(60)
                )) => {}
            }
        }
    }

    async fn run_upload(&self, cancel: CancellationToken) {
        loop {
            if cancel.is_cancelled() {
                break;
            }

            match self.upload_once(cancel.clone()).await {
                Ok(true) => {
                    // Còn dữ liệu chưa upload thành công.
                    tokio::select! {
                        _ = cancel.cancelled() => break,
                        _ = self.upload_notify.notified() => {},
                        _ = sleep(Duration::from_secs(5)) => {},
                    }
                }
                Ok(false) => {
                    // Không còn dữ liệu cần upload.
                    // Chờ thông báo để tránh truy vấn SQLite liên tục.
                    tokio::select! {
                        _ = cancel.cancelled() => break,
                        _ = self.upload_notify.notified() => {},
                    }
                }
                Err(error) => {
                    eprintln!("[tax-sync] Upload iteration failed: {error:#}");

                    // Lỗi tổng thể: thử lại sau 30 giây,
                    // hoặc chạy ngay khi có thông báo mới.
                    tokio::select! {
                        _ = cancel.cancelled() => break,
                        _ = self.upload_notify.notified() => {},
                        _ = sleep(Duration::from_secs(30)) => {},
                    }
                }
            }
        }
    }

    async fn run_update(&self, cancel: CancellationToken) {
        loop {
            if cancel.is_cancelled() {
                break;
            }

            // Kiểm tra database ngay khi khởi động,
            // sau đó chờ thông báo hoặc đến lịch retry.
            match self.update_once(cancel.clone()).await {
                Ok(true) => {
                    // Vẫn còn dữ liệu chưa xử lý thành công.
                    tokio::select! {
                        _ = cancel.cancelled() => break,
                        _ = sleep(Duration::from_secs(5)) => {},
                        _ = self.update_notify.notified() => {},
                    }
                }
                Ok(false) => {
                    // Không còn công việc: chỉ chờ thông báo.
                    tokio::select! {
                        _ = cancel.cancelled() => break,
                        _ = self.update_notify.notified() => {},
                    }
                }
                Err(error) => {
                    eprintln!("[tax-sync] Update iteration failed: {error:#}");

                    // Lỗi truy vấn hoặc lỗi xử lý tổng thể:
                    // thử lại sau 30 giây.
                    tokio::select! {
                        _ = cancel.cancelled() => break,
                        _ = sleep(Duration::from_secs(30)) => {},
                        _ = self.update_notify.notified() => {},
                    }
                }
            }
        }
    }

    async fn upload_once(&self, cancel: CancellationToken) -> Result<bool> {
        if cancel.is_cancelled() {
            return Ok(false);
        }

        let accounts = self.db.get_enabled_tax_accounts(false)?;

        for account in accounts {
            if cancel.is_cancelled() {
                return Ok(false);
            }

            if !account.auto_sync {
                continue;
            }

            let invoices = self.db.get_invoices_pending_upload(&account.tax_code)?;

            if invoices.is_empty() {
                continue;
            }

            match self.ensure_valid_app_token(&account).await {
                Ok(Some(token)) => {
                    if cancel.is_cancelled() {
                        return Ok(false);
                    }

                    let type_app = TypeApp::try_from(account.type_app.as_str())?;

                    self.invoices_pending_upload(
                        &account.tax_code,
                        &invoices,
                        &token,
                        &type_app,
                        cancel.clone(),
                    )
                    .await?;

                    eprintln!(
                        "[tax-sync] App token ready: tax_code={}, app={}",
                        account.tax_code, account.type_app
                    );
                }
                Ok(None) => {}
                Err(error) => {
                    if cancel.is_cancelled() {
                        return Ok(false);
                    }

                    eprintln!(
                        "[tax-sync] App authentication failed: tax_code={}, error={:#}",
                        account.tax_code, error
                    );
                }
            }
        }

        if cancel.is_cancelled() {
            return Ok(false);
        }

        // Kiểm tra lại database sau khi xử lý.
        let has_pending = self.db.has_pending_uploads()?;

        Ok(has_pending)
    }

    /// Một lượt quét toàn bộ tài khoản thuế đang bật.
    pub async fn run_once(&self, cancel: CancellationToken) -> Result<()> {
        let accounts_error = self.db.get_enabled_tax_accounts(true)?;
        let accounts = self.db.get_enabled_tax_accounts(false)?;
        self.observer.on_notification(&accounts_error);

        for account in accounts {
            if cancel.is_cancelled() {
                break;
            }

            if let Err(err) = self.db.update_sync_time(&account.tax_code) {
                eprintln!("[tax-sync] Update sync time failed: {err}");
            }

            if let Err(error) = self.process_account(&account, cancel.clone()).await {
                eprintln!("[tax-sync] Account {} failed: {error:#}", account.tax_code);
            }
        }

        Ok(())
    }

    pub async fn update_once(&self, cancel: CancellationToken) -> Result<bool> {
        if cancel.is_cancelled() {
            return Ok(false);
        }

        let accounts = self.db.get_enabled_tax_accounts(false)?;

        for account in accounts {
            if cancel.is_cancelled() {
                return Ok(false);
            }

            if let Err(error) = self.process_account_update(&account, cancel.clone()).await {
                if cancel.is_cancelled() {
                    return Ok(false);
                }

                eprintln!(
                    "[tax-sync] Account {} update failed: {error:#}",
                    account.tax_code
                );
            }
        }

        // Kiểm tra lại trạng thái thực tế sau khi xử lý.
        let has_pending = self.db.has_pending_updates()?;

        Ok(has_pending)
    }

    async fn process_account_update(
        &self,
        account: &TaxAccount,
        cancel: CancellationToken,
    ) -> Result<()> {
        if cancel.is_cancelled() {
            return Ok(());
        }

        let invoice_types = parse_invoice_types(&account.invoice_type)?;

        if invoice_types.is_empty() {
            return Ok(());
        }

        // Không bắt đầu đăng nhập nếu worker đã được yêu cầu dừng.
        if cancel.is_cancelled() {
            return Ok(());
        }

        let token = self.ensure_valid_token(account).await?;

        // Nếu đăng nhập xong mới nhận yêu cầu dừng, không xử lý tiếp.
        if cancel.is_cancelled() {
            return Ok(());
        }

        let service = TaxSyncService::new(
            self.db.clone(),
            self.upload_notify.clone(),
            self.update_notify.clone(),
            self.captcha_solver.clone(),
        )?;

        for invoice_type in &invoice_types {
            if cancel.is_cancelled() {
                break;
            }

            let (completed, failed) = service
                .retry_pending_details(
                    &account.tax_code,
                    *invoice_type,
                    &token,
                    self.config.delay_ms,
                    1000, // Số hóa đơn mỗi batch
                    cancel.clone(),
                )
                .await?;

            println!(
                "[tax-sync] Detail retry: tax_code={}, type={}, completed={}, failed={}",
                account.tax_code, invoice_type, completed, failed
            );

            if cancel.is_cancelled() {
                break;
            }
        }

        Ok(())
    }

    async fn process_account(&self, account: &TaxAccount, cancel: CancellationToken) -> Result<()> {
        // Không bắt đầu xử lý tài khoản mới nếu đã có yêu cầu dừng.
        if cancel.is_cancelled() {
            return Ok(());
        }

        let invoice_types = parse_invoice_types(&account.invoice_type)?;

        if invoice_types.is_empty() {
            return Ok(());
        }

        // Đăng nhập có thể mất thời gian.
        // Nếu dừng trong lúc đăng nhập, chờ đăng nhập xong rồi thoát.
        let token = self.ensure_valid_token(account).await?;

        if cancel.is_cancelled() {
            return Ok(());
        }

        self.seed_days(account, &invoice_types).await?;

        if cancel.is_cancelled() {
            return Ok(());
        }

        for invoice_type in &invoice_types {
            if cancel.is_cancelled() {
                break;
            }

            if cancel.is_cancelled() {
                break;
            }

            let dates = self
                .db
                .get_incomplete_days(&account.tax_code, *invoice_type)?;

            for date in dates {
                if cancel.is_cancelled() {
                    break;
                }

                eprintln!(
                    "[tax-sync] tax_code={}, type={}, date={}",
                    account.tax_code, invoice_type, date
                );

                // Cho phép dừng ngay cả khi đang chờ delay.
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = request_delay(self.config.delay_ms) => {}
                }

                if cancel.is_cancelled() {
                    break;
                }

                if let Err(error) = self
                    .sync_one_day(account, *invoice_type, &date, &token, cancel.clone())
                    .await
                {
                    eprintln!(
                        "[tax-sync] tax_code={}, type={}, date={}: {error:#}",
                        account.tax_code, invoice_type, date
                    );
                }

                // Nếu vừa hoàn thành một ngày mà nhận tín hiệu dừng,
                // không tiếp tục sang ngày tiếp theo.
                if cancel.is_cancelled() {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Tạo các bản ghi ngày từ start_date đến hôm nay.
    ///
    /// ON CONFLICT DO NOTHING trong ensure_sync_day giúp giữ nguyên
    /// trạng thái của các ngày đã được xử lý trước đó.
    async fn seed_days(&self, account: &TaxAccount, invoice_types: &[u8]) -> Result<()> {
        let start = NaiveDate::parse_from_str(&account.start_date, "%Y-%m-%d")?;

        let today = vietnam_today().await?;

        if start > today {
            // Chưa đến ngày cần đồng bộ, không tạo ngày.
            return Ok(());
        }

        let mut date = start;

        loop {
            let date_string = date.format("%Y-%m-%d").to_string();

            for invoice_type in invoice_types {
                self.db
                    .ensure_sync_day(&account.tax_code, *invoice_type, &date_string)?;
            }

            if date >= today {
                break;
            }

            date = date
                .succ_opt()
                .ok_or_else(|| anyhow!("Không thể tăng ngày đồng bộ"))?;
        }

        Ok(())
    }

    async fn ensure_valid_token(&self, account: &TaxAccount) -> Result<String> {
        if let (Some(token), Some(expires_at)) = (
            account.token.as_deref(),
            account.token_expires_at.as_deref(),
        ) {
            if !token.trim().is_empty() && token_is_valid(expires_at) {
                return Ok(token.to_string());
            }
        }
        let store = SecretStore::new();
        let password = store.get_secret(&account.password_key).map_err(|error| {
            anyhow!(
                "Không đọc được mật khẩu Keychain cho tài khoản {}: {error:#}",
                account.tax_code
            )
        })?;

        let solver = Arc::clone(&self.captcha_solver);

        let login = self
            .auth
            .login(account, &password, move |captcha_content| {
                let solver = Arc::clone(&solver);
                let svg = captcha_content.to_string();

                async move { solver.solve(&svg).map_err(TaxSyncError::Authentication) }
            })
            .await
            .map_err(|error| {
                let message = format!(
                    "Đăng nhập cơ quan thuế thất bại cho {}: {error}",
                    account.tax_code
                );

                if let Err(db_error) = self.db.set_auth_failure(&account.tax_code, &message) {
                    eprintln!(
                        "Không thể cập nhật trạng thái đăng nhập thất bại cho {}: {}",
                        account.tax_code, db_error
                    );
                }

                anyhow!("{}", message)
            })?;

        let lifetime = login
            .expires_in
            .unwrap_or((TOKEN_LIFETIME_HOURS * 60 * 60) as u64);

        let expires_at = (Utc::now() + ChronoDuration::seconds(lifetime as i64)).to_rfc3339();

        // Cần bổ sung phương thức này trong database.rs.
        self.db
            .set_auth_success(&account.tax_code, &login.token, &expires_at)?;

        Ok(login.token)
    }

    async fn ensure_valid_app_token(
        &self,
        account: &TaxAccount,
    ) -> Result<Option<ApplicationLoginResponse>> {
        // Chỉ xử lý token ứng dụng khi bật đồng bộ lên ERP.
        if !account.auto_sync {
            return Ok(None);
        }

        // Token ứng dụng còn hạn thì sử dụng lại.
        if let (Some(token), Some(expires_at)) = (
            account.app_token.as_deref(),
            account.app_token_expires_at.as_deref(),
        ) {
            if !token.trim().is_empty() && token_is_valid(expires_at) {
                return Ok(Some(ApplicationLoginResponse {
                    org_unit: account.org_unit.clone(),
                    year: Some(get_current_year()),
                    lang: Some("vi".to_string()),
                    org_unit_id: account.org_unit_id.clone(),
                    tenant_id: account.tenant_id.clone(),
                    access_token: token.to_string(),
                    expires_in: None,
                    refresh_token: account.app_refresh_token.clone(),
                    token_type: account
                        .app_token_type
                        .clone()
                        .unwrap_or_else(|| "Bearer".to_string()),
                    url_api: account.url_api.clone(),
                    org_units: vec![],
                }));
            }
        }
        let store = SecretStore::new();

        let password_key = account
            .pass_app_key
            .as_deref()
            .filter(|key| !key.trim().is_empty())
            .ok_or_else(|| {
                anyhow!(
                    "Chưa cấu hình Keychain password cho ứng dụng của tài khoản {}",
                    account.tax_code
                )
            })?;

        let password = store.get_secret(password_key).map_err(|error| {
            anyhow!(
                "Không đọc được mật khẩu ứng dụng từ Keychain cho tài khoản {}: {error:#}",
                account.tax_code
            )
        })?;

        // Thử refresh token trước nếu là vaOne.
        let login: ApplicationLoginResponse =
            if matches!(account.type_app.as_str(), "vaOne" | "vaShop") {
                if let Some(refresh_token) = account
                    .app_refresh_token
                    .as_deref()
                    .filter(|token| !token.trim().is_empty())
                {
                    let type_app = TypeApp::try_from(account.type_app.as_str())?;
                    match self
                        .auth
                        .refresh_vaone_token(
                            refresh_token,
                            &account.tenant_id,
                            &type_app,
                            &account.url_api,
                        )
                        .await
                    {
                        Ok(token) => token,
                        Err(error) => {
                            eprintln!(
                                "[tax-sync] Refresh app token failed, will login again: \
                         tax_code={}, error={:#}",
                                account.tax_code, error
                            );

                            self.login_application(account, &password).await?
                        }
                    }
                } else {
                    self.login_application(account, &password).await?
                }
            } else {
                self.login_application(account, &password).await?
            };

        // vaOnline không trả expires_in nên cần thời hạn mặc định phù hợp
        // với chính sách token thực tế của API vaOnline.
        let lifetime = login
            .expires_in
            .unwrap_or((TOKEN_LIFETIME_HOURS * 3600) as u64);

        let expires_at = (Utc::now() + ChronoDuration::seconds(lifetime as i64)).to_rfc3339();

        self.db.set_app_auth_success(
            &account.tax_code,
            &login.access_token,
            login.refresh_token.as_deref(),
            &login.token_type,
            &expires_at,
            login.org_unit_id.as_deref(),
            login.tenant_id.as_deref(),
        )?;

        Ok(Some(login))
    }

    async fn login_application(
        &self,
        account: &TaxAccount,
        password: &str,
    ) -> Result<ApplicationLoginResponse> {
        let type_app = TypeApp::try_from(account.type_app.as_str())?;
        self.auth
            .check_application_login(account, password, &type_app)
            .await
            .map_err(|error| {
                let message = format!(
                    "Đăng nhập ứng dụng {} thất bại cho {}: {error}",
                    account.type_app, account.tax_code
                );

                if let Err(db_error) = self.db.set_app_auth_failure(&account.tax_code, &message) {
                    eprintln!(
                        "Không thể cập nhật trạng thái đăng nhập ứng dụng cho {}: {}",
                        account.tax_code, db_error
                    );
                }

                anyhow::anyhow!("{}", message)
            })
    }

    async fn sync_one_day(
        &self,
        account: &TaxAccount,
        invoice_type: u8,
        date: &str,
        token: &str,
        cancel: CancellationToken,
    ) -> Result<()> {
        if cancel.is_cancelled() {
            return Ok(());
        }

        let service = TaxSyncService::new(
            self.db.clone(),
            self.upload_notify.clone(),
            self.update_notify.clone(),
            self.captcha_solver.clone(),
        )?;

        let request = SyncRequest {
            tax_code: account.tax_code.clone(),
            invoice_type,
            from_date: date.to_string(),
            to_date: date.to_string(),
            token: token.to_string(),
            delay_ms: self.config.delay_ms,
        };
        if cancel.is_cancelled() {
            return Ok(());
        }
        self.db
            .set_day_running(&account.tax_code, invoice_type, date)?;

        service
            .run(request, self.observer.as_ref(), cancel.clone())
            .await
            .map_err(|error| anyhow!(error.to_string()))?;

        if cancel.is_cancelled() {
            // Không đánh dấu completed nếu quá trình bị dừng.
            // Ngày cần được giữ ở trạng thái có thể retry.
            self.db
                .set_day_pending(&account.tax_code, invoice_type, date)?;
            return Ok(());
        }

        Ok(())
    }

    async fn invoices_pending_upload(
        &self,
        tax_code: &str,
        invoices: &Vec<InvoiceUpload>,
        token: &ApplicationLoginResponse,
        type_app: &TypeApp,
        cancel: CancellationToken,
    ) -> Result<Value> {
        if cancel.is_cancelled() {
            return Ok(Value::Array(Vec::new()));
        }

        // let invoices = self.db.get_invoices_pending_upload(tax_code)?;

        let mut results = Vec::new();
        const BATCH_SIZE: usize = 20;

        for batch in invoices.chunks(BATCH_SIZE) {
            // Không bắt đầu batch mới khi đã nhận yêu cầu dừng.
            if cancel.is_cancelled() {
                break;
            }

            // Bổ sung outOrIn và xml vào từng hóa đơn.
            let datas: Vec<Value> = batch
                .iter()
                .map(|item| {
                    let mut invoice = item.invoice.clone();

                    if let Value::Object(ref mut map) = invoice {
                        map.insert("outOrIn".to_string(), Value::from(item.invoice_type));

                        let xml_base64 = item
                            .xml_data
                            .as_ref()
                            .map(|xml| STANDARD.encode(xml.as_bytes()));

                        map.insert(
                            "xml".to_string(),
                            match xml_base64 {
                                Some(value) => Value::String(value),
                                None => Value::Null,
                            },
                        );
                    }

                    invoice
                })
                .collect();

            let payload = serde_json::json!({
                "data": datas
            });

            // Không select! để hủy ngang request upload.
            // Chờ kết quả để biết có cần cập nhật trạng thái DB hay không.
            match self.auth.post_data(&payload, token, type_app).await {
                Ok(result) => {
                    for invoice in batch {
                        if let Err(err) = self.db.set_upload_completed(
                            tax_code,
                            invoice.invoice_type,
                            &invoice.invoice_key,
                        ) {
                            eprintln!(
                                "[tax-sync] Mark upload completed failed: key={}, error={}",
                                invoice.invoice_key, err
                            );
                        }
                    }

                    println!("[tax-sync] Upload thành công: count={}", batch.len());

                    results.push(result);
                }

                Err(err) => {
                    let message = err.to_string();

                    for invoice in batch {
                        if let Err(db_err) = self.db.set_upload_failed(
                            tax_code,
                            invoice.invoice_type,
                            &invoice.invoice_key,
                            &message,
                        ) {
                            eprintln!(
                                "[tax-sync] Mark upload failed failed: key={}, error={}",
                                invoice.invoice_key, db_err
                            );
                        }
                    }

                    eprintln!(
                        "[tax-sync] Upload thất bại: count={}, error={}",
                        batch.len(),
                        message
                    );
                }
            }

            // Hoàn tất cập nhật trạng thái batch hiện tại rồi mới dừng.
            if cancel.is_cancelled() {
                break;
            }

            // Có thể dừng ngay trong thời gian chờ delay.
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = request_delay(self.config.delay_ms) => {}
            }
        }

        Ok(Value::Array(results))
    }
}

fn token_is_valid(expires_at: &str) -> bool {
    let Ok(expiry) = DateTime::parse_from_rfc3339(expires_at) else {
        return false;
    };

    // Đăng nhập lại sớm hơn 5 phút để tránh token hết hạn giữa lúc đồng bộ.
    let safety_margin = ChronoDuration::minutes(5);

    Utc::now() + safety_margin < expiry.with_timezone(&Utc)
}
