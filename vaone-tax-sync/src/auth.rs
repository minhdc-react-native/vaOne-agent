use std::future::Future;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::{Result, TaxSyncError},
    models::TaxAccount,
};

const CAPTCHA_URL: &str = "https://hoadondientu.gdt.gov.vn/api/captcha";
const LOGIN_URL: &str = "https://hoadondientu.gdt.gov.vn/api/security-taxpayer/authenticate";

/// Thời hạn token do ứng dụng quy ước: 24 giờ.
pub const TOKEN_EXPIRES_IN_SECS: u64 = 24 * 60 * 60;

#[derive(Debug, Clone)]
pub struct ApplicationLoginResponse {
    pub access_token: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub token_type: String,
}

#[derive(Debug, Deserialize)]
struct VaOnlineOrgUnit {
    id: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct VaOneTenantResponse {
    success: bool,
    #[serde(rename = "tenantId")]
    tenant_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VaOneTokenResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
    token_type: String,
}

#[derive(Debug, Deserialize)]
struct VaOneOrgUnit {
    code: String,
    #[serde(rename = "taxCode")]
    tax_code: String,
}

#[derive(Debug, Deserialize)]
struct CaptchaResponse {
    key: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
    ckey: &'a str,
    cvalue: &'a str,
}

#[derive(Debug, Clone)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: Option<u64>,
}

pub struct TaxAuthService {
    http: Client,
}

impl TaxAuthService {
    pub fn new() -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| TaxSyncError::Authentication(e.to_string()))?;

        Ok(Self { http })
    }

    /// Đăng nhập tài khoản thuế.
    ///
    /// `solve_captcha` nhận nội dung captcha và trả về đáp án.
    /// Mật khẩu được truyền riêng để có thể lấy từ macOS Keychain
    /// thay vì lưu trực tiếp trong SQLite.
    pub async fn login<F, Fut>(
        &self,
        account: &TaxAccount,
        password: &str,
        solve_captcha: F,
    ) -> Result<LoginResponse>
    where
        F: FnOnce(&str) -> Fut,
        Fut: Future<Output = Result<String>>,
    {
        // 1. Lấy captcha.
        let captcha_response = self
            .http
            .get(CAPTCHA_URL)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| TaxSyncError::Authentication(format!("Không thể lấy captcha: {e}")))?
            .error_for_status()
            .map_err(|e| {
                TaxSyncError::Authentication(format!("API captcha trả về HTTP lỗi: {e}"))
            })?;

        let captcha: CaptchaResponse = captcha_response.json().await.map_err(|e| {
            TaxSyncError::Authentication(format!("Không thể đọc response captcha: {e}"))
        })?;

        if captcha.key.trim().is_empty() || captcha.content.trim().is_empty() {
            return Err(TaxSyncError::Authentication(
                "API captcha trả về dữ liệu không hợp lệ".to_string(),
            ));
        }

        // 2. Giải captcha bằng hàm do ứng dụng cung cấp.
        let captcha_value = solve_captcha(&captcha.content).await?;
        let captcha_value = captcha_value.trim();

        if captcha_value.is_empty() {
            return Err(TaxSyncError::Authentication(
                "Đáp án captcha rỗng".to_string(),
            ));
        }

        // 3. Gửi yêu cầu đăng nhập.
        let payload = LoginRequest {
            username: &account.username,
            password,
            ckey: &captcha.key,
            cvalue: captcha_value,
        };
        // return Err(TaxSyncError::Authentication(format!(
        //     "payload>> {:#?}",
        //     payload
        // )));

        let response = self
            .http
            .post(LOGIN_URL)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("request-id", uuid::Uuid::new_v4().to_string())
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                TaxSyncError::Authentication(format!("Không thể gửi yêu cầu đăng nhập: {e}"))
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|e| {
            TaxSyncError::Authentication(format!("Không thể đọc response đăng nhập: {e}"))
        })?;

        if !status.is_success() {
            return Err(TaxSyncError::Authentication(format!(
                "Đăng nhập thất bại, HTTP {LOGIN_URL}: {} : {:#?}",
                truncate_body(&body),
                payload
            )));
        }

        // 4. Đọc token.
        // Hỗ trợ response dạng {"token":"..."},
        // {"access_token":"..."} hoặc chuỗi JSON "token...".
        let token = parse_token(&body)?;

        // 5. API không trả expires_in, nên dùng mặc định 24 giờ.
        Ok(LoginResponse {
            token,
            expires_in: Some(TOKEN_EXPIRES_IN_SECS),
        })
    }

    pub async fn check_application_login(
        &self,
        account: &TaxAccount,
        password: &str,
    ) -> Result<ApplicationLoginResponse> {
        match account.type_app.as_str() {
            "vaOnline" => self.check_vaonline_login(account, password).await,
            "vaOne" => self.check_vaone_login(account, password).await,
            _ => Err(TaxSyncError::Authentication(
                "Loại ứng dụng không được hỗ trợ".to_string(),
            )),
        }
    }

    //Kiểm tra vaOnline
    pub async fn check_vaonline_login(
        &self,
        account: &TaxAccount,
        password: &str,
    ) -> Result<ApplicationLoginResponse> {
        let access_code = account.access_code.as_deref().unwrap_or("").trim();

        let username = account.user_app.as_deref().unwrap_or("").trim();

        let org_unit = account.org_unit.as_deref().unwrap_or("").trim();

        if access_code.is_empty() || username.is_empty() || org_unit.is_empty() {
            return Err(TaxSyncError::Authentication(
                "Thiếu mã truy cập, tài khoản ứng dụng hoặc mã đơn vị".to_string(),
            ));
        }

        // 1. Đăng nhập ứng dụng.
        let login_url = format!("https://{}.vaonline.vn/api/User/Login", access_code);

        let response = self
            .http
            .post(&login_url)
            .header("Accept", "application/json")
            .form(&[
                ("username", username),
                ("pass", password),
                ("captcha_token", ""),
            ])
            .send()
            .await
            .map_err(|e| {
                TaxSyncError::Authentication(format!("Không thể đăng nhập vaOnline: {e}"))
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|e| {
            TaxSyncError::Authentication(format!("Không thể đọc response đăng nhập vaOnline: {e}"))
        })?;

        if !status.is_success() {
            return Err(TaxSyncError::Authentication(format!(
                "Đăng nhập vaOnline thất bại, HTTP {status}: {}",
                truncate_body(&body)
            )));
        }

        let token = parse_token(&body)?;

        // 2. Lấy danh sách đơn vị.
        let org_url = format!("https://{}.vaonline.vn/api/User/GetListDvcs", access_code);

        let response = self
            .http
            .get(&org_url)
            .header("Accept", "application/json")
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| {
                TaxSyncError::Authentication(format!(
                    "Không thể lấy danh sách đơn vị vaOnline: {e}"
                ))
            })?;

        let status = response.status();

        if !status.is_success() {
            return Err(TaxSyncError::Authentication(format!(
                "Lấy danh sách đơn vị vaOnline thất bại, HTTP {status}"
            )));
        }

        let units: Vec<VaOnlineOrgUnit> = response.json().await.map_err(|e| {
            TaxSyncError::Authentication(format!(
                "Dữ liệu danh sách đơn vị vaOnline không hợp lệ: {e}"
            ))
        })?;

        // 3. Kiểm tra mã đơn vị.
        if !units.iter().any(|unit| unit.id == org_unit) {
            return Err(TaxSyncError::Authentication(format!(
                "Không tìm thấy đơn vị vaOnline có mã: {org_unit}"
            )));
        }

        Ok(ApplicationLoginResponse {
            access_token: token,
            expires_in: None,
            refresh_token: None,
            token_type: "Bearer".to_string(),
        })
    }

    //check vaOne online
    pub async fn check_vaone_login(
        &self,
        account: &TaxAccount,
        password: &str,
    ) -> Result<ApplicationLoginResponse> {
        const BASE_URL: &str = "http://10.8.0.146:30334";
        const CLIENT_ID: &str = "VaOneApi_Web";
        const CLIENT_SECRET: &str = "1q2w3e*";

        let access_code = account.access_code.as_deref().unwrap_or("").trim();

        let username = account.user_app.as_deref().unwrap_or("").trim();

        let org_unit = account.org_unit.as_deref().unwrap_or("").trim();

        let tax_code = account.tax_code.trim();

        if access_code.is_empty() || username.is_empty() || org_unit.is_empty() {
            return Err(TaxSyncError::Authentication(
                "Thiếu mã truy cập, tài khoản ứng dụng hoặc mã đơn vị".to_string(),
            ));
        }

        // 1. Kiểm tra tenant.
        let tenant_url = format!(
            "{BASE_URL}/api/abp/multi-tenancy/tenants/by-name/{}",
            access_code
        );

        let response = self
            .http
            .get(&tenant_url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                TaxSyncError::Authentication(format!("Không thể kiểm tra tenant vaOne: {e}"))
            })?;

        let status = response.status();

        if !status.is_success() {
            return Err(TaxSyncError::Authentication(format!(
                "Kiểm tra mã truy cập vaOne thất bại, HTTP {status}"
            )));
        }

        let tenant: VaOneTenantResponse = response.json().await.map_err(|e| {
            TaxSyncError::Authentication(format!("Dữ liệu tenant vaOne không hợp lệ: {e}"))
        })?;

        if !tenant.success || tenant.tenant_id.as_deref().unwrap_or("").is_empty() {
            return Err(TaxSyncError::Authentication(format!(
                "Mã truy cập vaOne không hợp lệ: {access_code}"
            )));
        }

        // 2. Đăng nhập lấy access_token và refresh_token.
        let token_url = format!("{BASE_URL}/connect/token");

        let response = self
            .http
            .post(&token_url)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("client_secret", CLIENT_SECRET),
                ("grant_type", "password"),
                ("username", username),
                ("password", password),
                ("scope", "VaOneApi offline_access"),
            ])
            .send()
            .await
            .map_err(|e| TaxSyncError::Authentication(format!("Không thể đăng nhập vaOne: {e}")))?;

        let status = response.status();
        let body = response.text().await.map_err(|e| {
            TaxSyncError::Authentication(format!("Không thể đọc response đăng nhập vaOne: {e}"))
        })?;

        if !status.is_success() {
            return Err(TaxSyncError::Authentication(format!(
                "Đăng nhập vaOne thất bại, HTTP {status}: {}",
                truncate_body(&body)
            )));
        }

        let token: VaOneTokenResponse = serde_json::from_str(&body).map_err(|e| {
            TaxSyncError::Authentication(format!("Response token vaOne không hợp lệ: {e}"))
        })?;

        if token.access_token.trim().is_empty() {
            return Err(TaxSyncError::Authentication(
                "vaOne không trả về access_token hợp lệ".to_string(),
            ));
        }

        // 3. Lấy danh sách đơn vị bằng access_token.
        let units_url = format!("{BASE_URL}/api/app/org-unit/my-list");

        let response = self
            .http
            .get(&units_url)
            .header("Accept", "application/json")
            .bearer_auth(&token.access_token)
            .send()
            .await
            .map_err(|e| {
                TaxSyncError::Authentication(format!("Không thể lấy danh sách đơn vị vaOne: {e}"))
            })?;

        let status = response.status();

        if !status.is_success() {
            return Err(TaxSyncError::Authentication(format!(
                "Lấy danh sách đơn vị vaOne thất bại, HTTP {status}"
            )));
        }

        let units: Vec<VaOneOrgUnit> = response.json().await.map_err(|e| {
            TaxSyncError::Authentication(format!(
                "Dữ liệu danh sách đơn vị vaOne không hợp lệ: {e}"
            ))
        })?;

        // 4. Đơn vị phải khớp cả mã đơn vị và mã số thuế.
        let matched = units
            .iter()
            .any(|unit| unit.code == org_unit && unit.tax_code == tax_code);

        if !matched {
            return Err(TaxSyncError::Authentication(format!(
                "Không tìm thấy đơn vị '{org_unit}' thuộc mã số thuế '{tax_code}'"
            )));
        }

        Ok(ApplicationLoginResponse {
            access_token: token.access_token,
            expires_in: Some(token.expires_in),
            refresh_token: token.refresh_token,
            token_type: token.token_type,
        })
    }

    pub async fn refresh_vaone_token(
        &self,
        refresh_token: &str,
    ) -> Result<ApplicationLoginResponse> {
        const BASE_URL: &str = "http://10.8.0.146:30334";
        const CLIENT_ID: &str = "VaOneApi_Web";
        const CLIENT_SECRET: &str = "1q2w3e*";

        let refresh_token = refresh_token.trim();

        if refresh_token.is_empty() {
            return Err(TaxSyncError::Authentication(
                "Refresh token không được để trống".to_string(),
            )
            .into());
        }

        let url = format!("{BASE_URL}/connect/token");

        let response = self
            .http
            .post(&url)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("client_secret", CLIENT_SECRET),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
            ])
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(TaxSyncError::Authentication(format!(
                "Refresh token vaOne thất bại (HTTP {}): {}",
                status.as_u16(),
                body
            ))
            .into());
        }

        let token: VaOneTokenResponse = serde_json::from_str(&body).map_err(|error| {
            TaxSyncError::Authentication(format!(
                "Không đọc được phản hồi refresh token vaOne: {error}"
            ))
        })?;

        if token.access_token.trim().is_empty() {
            return Err(TaxSyncError::Authentication(
                "vaOne không trả về access_token".to_string(),
            )
            .into());
        }

        Ok(ApplicationLoginResponse {
            access_token: token.access_token,
            expires_in: Some(token.expires_in),

            // Giữ refresh token cũ nếu server không trả token mới.
            refresh_token: token
                .refresh_token
                .filter(|value| !value.trim().is_empty())
                .or_else(|| Some(refresh_token.to_string())),

            // token_type là String, không phải Option<String>.
            token_type: if token.token_type.trim().is_empty() {
                "Bearer".to_string()
            } else {
                token.token_type
            },
        })
    }
}

/// Đọc token từ response JSON.
fn parse_token(body: &str) -> Result<String> {
    let value: Value = serde_json::from_str(body).map_err(|_| {
        TaxSyncError::Authentication("Response đăng nhập không phải JSON hợp lệ".to_string())
    })?;

    let token = match &value {
        Value::String(token) => Some(token.as_str()),
        Value::Object(object) => object
            .get("token")
            .or_else(|| object.get("access_token"))
            .and_then(Value::as_str),
        _ => None,
    };

    let token = token
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| {
            TaxSyncError::Authentication(format!(
                "Response đăng nhập không chứa token hợp lệ: {}",
                truncate_body(body)
            ))
        })?;

    Ok(token.to_string())
}

/// Giới hạn nội dung lỗi để tránh log response quá dài.
fn truncate_body(body: &str) -> String {
    const MAX_CHARS: usize = 500;

    let mut chars = body.chars();
    let truncated: String = chars.by_ref().take(MAX_CHARS).collect();

    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}
