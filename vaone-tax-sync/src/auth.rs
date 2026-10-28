use std::future::Future;
use std::time::Duration;

use chrono::Datelike;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::{Result, TaxSyncError},
    models::{Captcha, TaxAccount},
};

const CAPTCHA_URL: &str = "https://hoadondientu.gdt.gov.vn/api/captcha";
const LOGIN_URL: &str = "https://hoadondientu.gdt.gov.vn/api/security-taxpayer/authenticate";
const PROFILE_URL: &str = "https://hoadondientu.gdt.gov.vn/api/security-taxpayer/profile";
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeApp {
    VaOnline,
    VaOne,
    VaShop,
}
impl TryFrom<&str> for TypeApp {
    type Error = TaxSyncError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "vaOne" => Ok(Self::VaOne),
            "vaShop" => Ok(Self::VaShop),
            "vaOnline" => Ok(Self::VaOnline),
            _ => Err(TaxSyncError::InvalidConfig(format!(
                "type_app không hợp lệ: {}",
                value
            ))),
        }
    }
}

/// Thời hạn token do ứng dụng quy ước: 24 giờ.
pub const TOKEN_EXPIRES_IN_SECS: u64 = 24 * 60 * 60;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct VaOnlineOrgUnit {
    id: String,
    value: String,
    tax_code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplicationLoginResponse {
    pub org_unit: Option<String>,
    pub year: Option<String>,
    pub lang: Option<String>,
    pub org_unit_id: Option<String>,
    pub tenant_id: Option<String>,
    pub access_token: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub url_api: Option<String>,
    pub org_units: Vec<VaOnlineOrgUnit>,
}

#[derive(Debug, Deserialize)]
struct VaOnlineLogin {
    ok: bool,
    token: Option<String>,
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
    id: String,
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

#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: Option<u64>,
    pub name: String,
}

pub async fn get_captcha(http: &Client) -> Result<Captcha> {
    let response = http
        .get(CAPTCHA_URL)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| TaxSyncError::Authentication(format!("Không thể lấy captcha: {e}")))?
        .error_for_status()
        .map_err(|e| TaxSyncError::Authentication(format!("API captcha trả về HTTP lỗi: {e}")))?;

    let captcha: CaptchaResponse = response.json().await.map_err(|e| {
        TaxSyncError::Authentication(format!("Không thể đọc response captcha: {e}"))
    })?;

    if captcha.key.trim().is_empty() || captcha.content.trim().is_empty() {
        return Err(TaxSyncError::Authentication(
            "API captcha trả về dữ liệu không hợp lệ".to_string(),
        ));
    }

    Ok(Captcha {
        ckey: captcha.key,
        cvalue: captcha.content,
    })
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
        let captcha = get_captcha(&self.http).await?;

        // Giải captcha bằng hàm do ứng dụng cung cấp.
        let captcha_value = solve_captcha(&captcha.cvalue).await?;
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
            ckey: &captcha.ckey,
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
        let profile = self
            .http
            .get(PROFILE_URL)
            .bearer_auth(&token)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("request-id", uuid::Uuid::new_v4().to_string())
            .send()
            .await
            .map_err(|e| {
                TaxSyncError::Authentication(format!("Không thể lấy thông tin profile: {e}"))
            })?
            .error_for_status()
            .map_err(|e| TaxSyncError::Authentication(format!("Lấy profile thất bại: {e}")))?;

        let profile_json: serde_json::Value = profile.json().await.map_err(|e| {
            TaxSyncError::Authentication(format!("Không thể đọc dữ liệu profile: {e}"))
        })?;

        let name = profile_json
            .get("name")
            .and_then(|value| value.as_str())
            .ok_or_else(|| {
                TaxSyncError::Authentication("Profile không có trường name hợp lệ".to_string())
            })?;

        Ok(LoginResponse {
            token,
            name: name.to_string(),
            expires_in: Some(TOKEN_EXPIRES_IN_SECS),
        })
    }

    pub async fn check_application_login(
        &self,
        account: &TaxAccount,
        password: &str,
        type_app: &TypeApp,
    ) -> Result<ApplicationLoginResponse> {
        match account.type_app.as_str() {
            "vaOnline" => self.check_vaonline_login(account, password).await,
            "vaOne" => self.check_vaone_login(account, password, type_app).await,
            "vaShop" => self.check_vaone_login(account, password, type_app).await,
            _ => Err(TaxSyncError::Authentication(
                "Loại ứng dụng không được hỗ trợ".to_string(),
            )),
        }
    }

    //Kiểm tra vaOnline
    async fn check_vaonline_login(
        &self,
        account: &TaxAccount,
        password: &str,
    ) -> Result<ApplicationLoginResponse> {
        let access_code = account.access_code.as_deref().unwrap_or("").trim();

        let username = account.user_app.as_deref().unwrap_or("").trim();

        let mut org_unit = account.org_unit.as_deref().unwrap_or("").trim().to_string();

        if access_code.is_empty() || username.is_empty() {
            return Err(TaxSyncError::Authentication(
                "Thiếu mã truy cập hoặc tài khoản ứng dụng".to_string(),
            ));
        }

        let base_url = account
            .url_api
            .clone()
            .unwrap_or_else(|| format!("https://{}.vaonline.vn", access_code));
        // 1. Đăng nhập ứng dụng.
        let login_url = format!("{base_url}/api/User/Login");

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

        if !status.is_success() {
            return Err(TaxSyncError::Authentication(format!(
                "Đăng nhập vaOnline thất bại, HTTP {status}"
            )));
        }

        let login: VaOnlineLogin = response.json().await.map_err(|e| {
            TaxSyncError::Authentication(format!("Dữ liệu vaOnline không hợp lệ: {e}"))
        })?;

        if !login.ok {
            return Err(TaxSyncError::Authentication(
                "Sai thông tin truy cập vaOnline".to_string(),
            ));
        }

        let token = login.token.ok_or_else(|| {
            TaxSyncError::Authentication("vaOnline không trả về token".to_string())
        })?;
        if org_unit.is_empty() {
            // Lấy tạm 1 đơn vị
            let org_url = format!("{base_url}/api/User/GetListDvcs");
            let access_token = format!("{};;{};{}", token, get_current_year(), "vi");

            let response = self
                .http
                .get(&org_url)
                .header("Accept", "application/json")
                .header("x-tenant-name", access_code)
                .bearer_auth(&access_token)
                .send()
                .await
                .map_err(|e| {
                    TaxSyncError::Authentication(format!(
                        "Không thể lấy danh sách đơn vị lần 1 vaOnline: {e}"
                    ))
                })?;

            let status = response.status();

            if !status.is_success() {
                return Err(TaxSyncError::Authentication(format!(
                    "Lấy danh sách đơn vị lần 1 vaOnline thất bại, HTTP {status}"
                )));
            }

            let result: serde_json::Value = response.json().await.map_err(|e| {
                TaxSyncError::Authentication(format!(
                    "Không thể đọc dữ liệu danh sách đơn vị lần 1 vaOnline: {e}"
                ))
            })?;

            org_unit = result
                .as_array()
                .and_then(|items| items.first())
                .and_then(|item| item.get("id"))
                .and_then(|id| {
                    id.as_str()
                        .map(String::from)
                        .or_else(|| id.as_i64().map(|n| n.to_string()))
                })
                .ok_or_else(|| {
                    TaxSyncError::Authentication(
                        "Không tìm thấy id của đơn vị đầu tiên trong danh sách vaOnline"
                            .to_string(),
                    )
                })?;
        }
        // 2. Lấy danh sách đơn vị.
        let org_url = format!("{base_url}/api/System/GetDataByWindowNo");
        let payload = serde_json::json!({
            "window_id": "WIN00115",
            "start": 0,
            "count": 100,
            "continue": "true",
            "filter": [
                {
                    "columnName": "MS_THUE",
                    "columnType": "VC_CODE_SM",
                    "value": account.tax_code
                },
                {
                    "columnName": "TEN_V",
                    "columnType": "VC_INFO_LG",
                    "value": ""
                }
            ],
            "infoparam": null,
            "tlbparam": []
        });
        let access_token = format!("{};{};{};{}", token, org_unit, get_current_year(), "vi");
        let response = self
            .http
            .post(&org_url)
            .header("Accept", "application/json")
            .header("x-tenant-name", access_code)
            .bearer_auth(&access_token)
            .json(&payload)
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

        let result: serde_json::Value = response.json().await.map_err(|e| {
            TaxSyncError::Authentication(format!(
                "Không thể đọc dữ liệu danh sách đơn vị vaOnline: {e}"
            ))
        })?;

        let units = result
            .get("data")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let org_units: Vec<VaOnlineOrgUnit> = units
            .iter()
            .map(|item| VaOnlineOrgUnit {
                id: item
                    .get("DVCS_ID")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                value: item
                    .get("TEN_V")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                tax_code: item
                    .get("MS_THUE")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
            })
            .collect();

        // 3. Kiểm tra mã đơn vị.
        let unit = org_units.iter().find(|unit| unit.id == org_unit);

        Ok(ApplicationLoginResponse {
            org_unit: Some(org_unit.to_string()),
            year: Some(get_current_year()),
            lang: Some("vi".to_string()),
            org_unit_id: unit.map(|unit| unit.id.clone()),
            tenant_id: Some(access_code.to_string()),
            access_token: token,
            expires_in: None,
            refresh_token: None,
            token_type: "Bearer".to_string(),
            url_api: account.url_api.clone(),
            org_units,
        })
    }

    //check vaOne online
    async fn check_vaone_login(
        &self,
        account: &TaxAccount,
        password: &str,
        type_app: &TypeApp,
    ) -> Result<ApplicationLoginResponse> {
        let default_url: &str = match type_app {
            TypeApp::VaOne => "http://10.8.0.146:30334",
            TypeApp::VaShop => "https://web.vashop.vn",
            _ => unreachable!(),
        };
        let base_url = account
            .url_api
            .as_deref()
            .filter(|url| !url.trim().is_empty())
            .unwrap_or(default_url);

        let client_id: &str = match type_app {
            TypeApp::VaOnline => "VaOnlineApi_App",
            TypeApp::VaOne => "VaOneApi_Web",
            TypeApp::VaShop => "VacomMartApi_App",
        };
        let scope: &str = match type_app {
            TypeApp::VaOnline => "VaOnlineApi offline_access",
            TypeApp::VaOne => "VaOneApi offline_access",
            TypeApp::VaShop => "VacomMartApi offline_access",
        };
        const CLIENT_SECRET: &str = "1q2w3e*";

        let access_code = account.access_code.as_deref().unwrap_or("").trim();

        let username = account.user_app.as_deref().unwrap_or("").trim();

        let org_unit = account.org_unit.as_deref().unwrap_or("").trim();

        let tax_code = account.tax_code.trim();

        if access_code.is_empty() || username.is_empty() {
            return Err(TaxSyncError::Authentication(
                "Thiếu mã truy cập hoặc tài khoản ứng dụng".to_string(),
            ));
        }

        // 1. Kiểm tra tenant.
        let tenant_url = format!(
            "{base_url}/api/abp/multi-tenancy/tenants/by-name/{}",
            access_code
        );
        let response = self
            .http
            .get(&tenant_url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| TaxSyncError::Authentication(format!("Không thể kiểm tra tenant: {e}")))?;

        let status = response.status();

        if !status.is_success() {
            return Err(TaxSyncError::Authentication(format!(
                "Kiểm tra mã truy cập thất bại, HTTP {status}"
            )));
        }

        let tenant: VaOneTenantResponse = response.json().await.map_err(|e| {
            TaxSyncError::Authentication(format!("Dữ liệu tenant không hợp lệ: {e}"))
        })?;

        if !tenant.success || tenant.tenant_id.as_deref().unwrap_or("").is_empty() {
            return Err(TaxSyncError::Authentication(format!(
                "Mã truy cập không hợp lệ: {access_code}"
            )));
        }
        let tenant_id: &str = tenant.tenant_id.as_deref().unwrap();

        // 2. Đăng nhập lấy access_token và refresh_token.
        let token_url = format!("{base_url}/connect/token");

        println!(
            "token_url={token_url}, client_id={client_id}, username={username}, scope={scope}"
        );

        let mut form = vec![
            ("client_id", client_id),
            ("grant_type", "password"),
            ("username", username),
            ("password", password),
            ("scope", scope),
        ];

        if matches!(type_app, TypeApp::VaOne) {
            form.push(("client_secret", CLIENT_SECRET));
        }

        let response = self
            .http
            .post(&token_url)
            .header("Accept", "application/json")
            .header("__tenant", tenant_id)
            .form(&form)
            .send()
            .await
            .map_err(|e| TaxSyncError::Authentication(format!("Không thể đăng nhập: {e}")))?;

        let status = response.status();
        let body = response.text().await.map_err(|e| {
            TaxSyncError::Authentication(format!("Không thể đọc response đăng nhập: {e}"))
        })?;

        if !status.is_success() {
            return Err(TaxSyncError::Authentication(format!(
                "Đăng nhập thất bại, HTTP {status}: {}",
                truncate_body(&body)
            )));
        }

        let token: VaOneTokenResponse = serde_json::from_str(&body).map_err(|e| {
            TaxSyncError::Authentication(format!("Response token không hợp lệ: {e}"))
        })?;

        if token.access_token.trim().is_empty() {
            return Err(TaxSyncError::Authentication(
                "Không trả về access_token hợp lệ".to_string(),
            ));
        }

        let org_units: Option<Vec<VaOnlineOrgUnit>> = match type_app {
            TypeApp::VaOne => {
                let default_url = "http://10.8.0.146:30334";

                let base_url = account
                    .url_api
                    .as_deref()
                    .filter(|url| !url.trim().is_empty())
                    .unwrap_or(default_url);

                let units_url = format!("{base_url}/api/app/org-unit/my-list");

                let response = self
                    .http
                    .get(&units_url)
                    .header("Accept", "application/json")
                    .bearer_auth(&token.access_token)
                    .send()
                    .await
                    .map_err(|e| {
                        TaxSyncError::Authentication(format!("Không thể lấy danh sách đơn vị: {e}"))
                    })?;

                let status = response.status();

                if !status.is_success() {
                    return Err(TaxSyncError::Authentication(format!(
                        "Lấy danh sách đơn vị thất bại, HTTP {status}"
                    )));
                }

                let units: Vec<Value> = response.json().await.map_err(|e| {
                    TaxSyncError::Authentication(format!(
                        "Dữ liệu danh sách đơn vị không hợp lệ: {e}"
                    ))
                })?;

                let items: Vec<VaOnlineOrgUnit> = units
                    .into_iter()
                    .map(|unit| VaOnlineOrgUnit {
                        id: unit
                            .get("code")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        value: unit
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        tax_code: unit
                            .get("taxCode")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                    })
                    .collect();

                Some(items)
            }
            _ => None,
        };

        let org_units: Vec<VaOnlineOrgUnit> = org_units.unwrap_or_default();

        let unit = org_units.iter().find(|unit| unit.id == org_unit);

        Ok(ApplicationLoginResponse {
            org_unit: Some(org_unit.to_string()),
            year: Some(get_current_year()),
            lang: Some("vi".to_string()),
            org_unit_id: unit.map(|unit| unit.id.clone()),
            tenant_id: Some(tenant.tenant_id.as_deref().unwrap_or("").to_string()),
            access_token: token.access_token,
            expires_in: Some(token.expires_in),
            refresh_token: token.refresh_token,
            token_type: token.token_type,
            url_api: account.url_api.clone(),
            org_units,
        })
    }

    pub async fn refresh_vaone_token(
        &self,
        refresh_token: &str,
        tenant_id: &Option<String>,
        type_app: &TypeApp,
        url_api: &Option<String>,
    ) -> Result<ApplicationLoginResponse> {
        let default_url: &str = match type_app {
            TypeApp::VaOne => "http://10.8.0.146:30334",
            TypeApp::VaShop => "https://web.vashop.vn",
            _ => unreachable!(),
        };
        let base_url = url_api
            .as_deref()
            .filter(|url| !url.trim().is_empty())
            .unwrap_or(default_url);

        let client_id: &str = match type_app {
            TypeApp::VaOnline => "VaOnlineApi_App",
            TypeApp::VaOne => "VaOneApi_Web",
            TypeApp::VaShop => "VacomMartApi_App",
        };
        const CLIENT_SECRET: &str = "1q2w3e*";

        let refresh_token = refresh_token.trim();

        if refresh_token.is_empty() {
            return Err(TaxSyncError::Authentication(
                "Refresh token không được để trống".to_string(),
            )
            .into());
        }

        let url = format!("{base_url}/connect/token");

        let mut request = self.http.post(&url).header("Accept", "application/json");

        if let Some(tenant_id) = tenant_id.as_deref() {
            request = request.header("__tenant", tenant_id);
        }

        let mut form = vec![
            ("client_id", client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        if matches!(type_app, TypeApp::VaOne) {
            form.push(("client_secret", CLIENT_SECRET));
        }

        let response = request.form(&form).send().await?;

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
            org_unit: None,
            year: None,
            lang: Some("vi".to_string()),
            org_unit_id: None,
            tenant_id: None,
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
            url_api: url_api.clone(),
            org_units: vec![],
        })
    }

    pub async fn post_data(
        &self,
        invoice: &Value,
        token: &ApplicationLoginResponse,
        type_app: &TypeApp,
    ) -> Result<Value> {
        let tenant_id = token
            .tenant_id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
            .ok_or_else(|| {
                TaxSyncError::Authentication("Không tìm thấy tenant_id hợp lệ".to_string())
            })?;

        let default_url: String = match type_app {
            TypeApp::VaOne => "http://10.8.0.146:30334".to_string(),
            TypeApp::VaShop => {
                "https://web.vashop.vn/api/app/tax-department/import-tax-department-data"
                    .to_string()
            }
            TypeApp::VaOnline => {
                format!(
                    "https://{}.vaonline.vn/api/TaxDepartment/ImportJsonData",
                    tenant_id
                )
            }
        };

        let url = token
            .url_api
            .as_deref()
            .filter(|url| !url.trim().is_empty())
            .unwrap_or(&default_url);

        let mut request = self.http.post(url).json(invoice);

        match type_app {
            TypeApp::VaOnline => {
                if let Some(org_unit) = &token.org_unit_id {
                    let access_token = format!(
                        "{};{};{};{}",
                        token.access_token,
                        org_unit,
                        get_current_year(),
                        "vi"
                    );
                    request = request.bearer_auth(access_token.clone());

                    request = request.header("x-tenant-name", tenant_id);

                    // eprintln!(
                    //     "URL ={} access_token={},tenant_id={:?}",
                    //     url, access_token, tenant_id
                    // );
                }
            }
            TypeApp::VaOne => {
                request = request.bearer_auth(&token.access_token);
                if let Some(org_unit_id) = &token.org_unit_id {
                    request = request.header("__orgid", org_unit_id);
                }

                request = request.header("__tenant", tenant_id);
            }
            TypeApp::VaShop => {
                request = request.bearer_auth(&token.access_token);
                request = request.header("__tenant", tenant_id);
            }
        }

        let response = request.send().await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            // println!(
            //     "Invoice payload: {}",
            //     serde_json::to_string_pretty(&invoice).unwrap_or_default()
            // );
            let message = format!(
                "Gửi dữ liệu hóa đơn thất bại ({}): {}",
                status.as_u16(),
                body
            );

            return match status.as_u16() {
                401 | 403 => Err(TaxSyncError::Authentication(message).into()),
                _ => Err(TaxSyncError::Api(message).into()),
            };
        }

        let result = if body.trim().is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&body).map_err(|e| {
                TaxSyncError::Authentication(format!(
                    "API trả về JSON không hợp lệ: {e}. Nội dung: {body}"
                ))
            })?
        };

        Ok(result)
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

pub fn get_current_year() -> String {
    chrono::Local::now().year().to_string()
}
