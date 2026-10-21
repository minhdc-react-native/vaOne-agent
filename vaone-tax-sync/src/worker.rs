use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, NaiveDateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::{
    auth::{get_current_year, ApplicationLoginResponse, TaxAuthService, TypeApp},
    client::request_delay,
    database::Database,
    error::TaxSyncError,
    models::{SyncRequest, TaxAccount},
    sync::{SyncObserver, TaxSyncService},
    SecretStore,
};

use tokio::time::sleep;

#[derive(Deserialize)]
struct TimeApiResponse {
    #[serde(rename = "dateTime")]
    date_time: String,
}

const DEFAULT_POLL_INTERVAL_SECS: u64 = 60;
const DEFAULT_DELAY_MS: u64 = 2 * 1000;
const TOKEN_LIFETIME_HOURS: i64 = 24;

fn parse_invoice_types(raw: &str) -> Result<Vec<u8>> {
    let types: Vec<u8> = serde_json::from_str(raw)?;

    if types.iter().any(|t| !(1..=4).contains(t)) {
        return Err(anyhow!("invoice_type chỉ chấp nhận các giá trị 1, 2, 3, 4"));
    }

    let mut unique = HashSet::new();

    Ok(types.into_iter().filter(|t| unique.insert(*t)).collect())
}

async fn vietnam_yesterday() -> Result<NaiveDate> {
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

    today
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
    cancel: Arc<AtomicBool>,
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
            cancel: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn start(self: Arc<Self>) {
        // Không khởi chạy nếu worker đang chạy.
        if self.cancel.swap(false, Ordering::SeqCst) {
            return;
        }

        // Reset cờ dừng trước khi chạy.
        self.cancel.store(false, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        if self.cancel.swap(true, Ordering::SeqCst) {
            return;
        }

        self.cancel.store(true, Ordering::SeqCst);
    }

    pub async fn run_workers(&self) {
        tokio::join!(self.run(), self.run_upload(),);
    }

    /// Chạy liên tục cho đến khi được yêu cầu dừng hoặc runtime kết thúc.
    async fn run(&self) {
        loop {
            if self.cancel.load(Ordering::Relaxed) {
                break;
            }

            if let Err(error) = self.run_once().await {
                eprintln!("[tax-sync] Worker iteration failed: {error:#}");
            }

            if self.cancel.load(Ordering::Relaxed) {
                break;
            }

            sleep(Duration::from_secs(self.config.poll_interval_secs.max(60))).await;
        }
    }

    async fn run_upload(&self) {
        loop {
            if self.cancel.load(Ordering::Relaxed) {
                break;
            }

            if let Err(error) = self.upload_once().await {
                eprintln!("[tax-sync] Upload iteration failed: {error:#}");
            }

            if self.cancel.load(Ordering::Relaxed) {
                break;
            }

            sleep(Duration::from_secs(5)).await;
        }
    }
    async fn upload_once(&self) -> Result<()> {
        let accounts = self.db.get_enabled_tax_accounts()?;
        for account in accounts {
            if self.cancel.load(Ordering::Relaxed) {
                break;
            }

            if account.auto_sync {
                match self.ensure_valid_app_token(&account).await {
                    Ok(Some(token)) => {
                        let type_app = TypeApp::try_from(account.type_app.as_str())?;
                        let _ = self
                            .invoices_pending_upload(&account.tax_code, &token, &type_app)
                            .await?;
                        eprintln!(
                            "[tax-sync] App token ready: tax_code={}, app={}",
                            account.tax_code, account.type_app
                        );
                    }
                    Ok(None) => {}
                    Err(error) => {
                        eprintln!(
                            "[tax-sync] App authentication failed: tax_code={}, error={:#}",
                            account.tax_code, error
                        );
                    }
                }
            }
        }

        Ok(())
    }
    /// Một lượt quét toàn bộ tài khoản thuế đang bật.
    pub async fn run_once(&self) -> Result<()> {
        let accounts = self.db.get_enabled_tax_accounts()?;
        for account in accounts {
            if self.cancel.load(Ordering::Relaxed) {
                break;
            }

            if let Err(error) = self.process_account(&account).await {
                eprintln!("[tax-sync] Account {} failed: {error:#}", account.tax_code);
            }
        }

        Ok(())
    }

    async fn process_account(&self, account: &TaxAccount) -> Result<()> {
        if self.cancel.load(Ordering::Relaxed) {
            return Ok(());
        }

        let invoice_types = parse_invoice_types(&account.invoice_type)?;

        if invoice_types.is_empty() {
            return Ok(());
        }

        let token = self.ensure_valid_token(account).await?;

        self.seed_days(account, &invoice_types).await?;

        for invoice_type in &invoice_types {
            if self.cancel.load(Ordering::Relaxed) {
                break;
            }

            let dates = self
                .db
                .get_incomplete_days(&account.tax_code, *invoice_type)?;

            for date in dates {
                if self.cancel.load(Ordering::Relaxed) {
                    break;
                }

                request_delay(self.config.delay_ms).await;

                if let Err(error) = self
                    .sync_one_day(account, *invoice_type, &date, &token)
                    .await
                {
                    eprintln!(
                        "[tax-sync] tax_code={}, type={}, date={}: {error:#}",
                        account.tax_code, invoice_type, date
                    );
                }
            }
        }

        // 2. Retry các hóa đơn chưa tải được detail.
        if self.cancel.load(Ordering::Relaxed) {
            return Ok(());
        }
        let service = TaxSyncService::new(self.db.clone())?;
        for invoice_type in &invoice_types {
            if self.cancel.load(Ordering::Relaxed) {
                break;
            }

            let (completed, failed) = service
                .retry_pending_details(
                    &account.tax_code,
                    *invoice_type,
                    &token,
                    self.config.delay_ms,
                    100, // Số hóa đơn mỗi batch
                    Arc::clone(&self.cancel),
                )
                .await?;

            println!(
                "[tax-sync] Detail retry: tax_code={}, type={}, completed={}, failed={}",
                account.tax_code, invoice_type, completed, failed
            );
        }

        Ok(())
    }

    /// Tạo các bản ghi ngày từ start_date đến hôm nay.
    ///
    /// ON CONFLICT DO NOTHING trong ensure_sync_day giúp giữ nguyên
    /// trạng thái của các ngày đã được xử lý trước đó.
    async fn seed_days(&self, account: &TaxAccount, invoice_types: &[u8]) -> Result<()> {
        let start = NaiveDate::parse_from_str(&account.start_date, "%Y-%m-%d")?;

        let yesterday = vietnam_yesterday().await?;

        if start > yesterday {
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

            if date >= yesterday {
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
                        .refresh_vaone_token(refresh_token, &type_app, &account.url_api)
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
    ) -> Result<()> {
        if self.cancel.load(Ordering::Relaxed) {
            return Ok(());
        }

        let service = TaxSyncService::new(self.db.clone())?;

        let request = SyncRequest {
            tax_code: account.tax_code.clone(),
            invoice_type,
            from_date: date.to_string(),
            to_date: date.to_string(),
            token: token.to_string(),
            delay_ms: self.config.delay_ms,
        };
        self.db
            .set_day_running(&account.tax_code, invoice_type, date)?;

        service
            .run(request, self.observer.as_ref(), Arc::clone(&self.cancel))
            .await
            .map_err(|error| anyhow!(error.to_string()))?;

        Ok(())
    }

    async fn invoices_pending_upload(
        &self,
        tax_code: &str,
        token: &ApplicationLoginResponse,
        type_app: &TypeApp,
    ) -> Result<Value> {
        let invoices = self.db.get_invoices_pending_upload(tax_code)?;

        let mut results = Vec::new();
        const BATCH_SIZE: usize = 1; // upload 5 hóa đơn 1 lần.

        for batch in invoices.chunks(BATCH_SIZE) {
            // Bổ sung outOrIn vào từng hóa đơn
            let datas: Vec<Value> = batch
                .iter()
                .map(|item| {
                    let mut invoice = item.invoice.clone();

                    if let Value::Object(ref mut map) = invoice {
                        map.insert("outOrIn".to_string(), Value::from(item.invoice_type));
                        // map.insert("xml".to_string(), Value::from(item.xml_data.clone()));
                        map.insert("xml".to_string(), Value::Null);
                    }

                    invoice
                })
                .collect();

            // Payload gửi lên API
            let payload = serde_json::json!({
                "data": datas
            });

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
                    println!(
                        "Invoice payload: {}",
                        serde_json::to_string_pretty(&payload).unwrap_or_default()
                    );

                    for invoice in batch {
                        let _ = self.db.set_upload_failed(
                            tax_code,
                            invoice.invoice_type,
                            &invoice.invoice_key,
                            &message,
                        );
                    }

                    eprintln!(
                        "[tax-sync] Upload thất bại: count={}, error={}",
                        batch.len(),
                        message
                    );
                }
            }

            request_delay(self.config.delay_ms).await;
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
