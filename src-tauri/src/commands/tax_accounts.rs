use serde::Deserialize;
use tauri::State;
use vaone_tax_sync::{
    auth::{LoginResponse, TaxAuthService, TypeApp},
    database::Database,
    error::TaxSyncError,
    models::{TaxAccount, TaxAccountInput},
    secret_store::SecretStore,
};

use crate::commands::system::predict_captcha;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckTaxLoginInput {
    pub account: TaxAccount,

    // Mật khẩu lấy trực tiếp từ form khi thêm mới.
    pub tax_password: Option<String>,
    pub app_password: Option<String>,
}

#[tauri::command]
pub fn get_tax_accounts(
    db: State<'_, Database>,
) -> Result<Vec<vaone_tax_sync::models::TaxAccount>, String> {
    db.get_tax_accounts().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tax_account(
    db: State<'_, Database>,
    input: TaxAccountInput,
) -> Result<TaxAccount, String> {
    db.create_tax_account(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_tax_account(
    db: State<'_, Database>,
    tax_code: String,
    input: TaxAccountInput,
) -> Result<TaxAccount, String> {
    db.update_tax_account(&tax_code, &input)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tax_account(db: State<'_, Database>, tax_code: String) -> Result<(), String> {
    db.delete_tax_account(&tax_code).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_tax_application_login(
    input: CheckTaxLoginInput,
) -> Result<LoginResponse, String> {
    let store = SecretStore::new();
    let auth = TaxAuthService::new().map_err(|e| e.to_string())?;
    let account = &input.account;

    // 1. Lấy mật khẩu cơ quan thuế:
    // Ưu tiên mật khẩu nhập trên form, nếu không có thì đọc SecretStore.
    let tax_password = match input.tax_password {
        Some(password) if !password.trim().is_empty() => password,
        _ => store.get_secret(&account.password_key).map_err(|e| {
            format!("Không lấy được mật khẩu cơ quan thuế. Vui lòng nhập lại mật khẩu: {e}")
        })?,
    };

    // 2. Đăng nhập cơ quan thuế, tự nhận diện captcha.
    let token = auth
        .login(account, &tax_password, |captcha_content| {
            let content = captcha_content.to_string();

            async move { predict_captcha(&content).map_err(TaxSyncError::Authentication) }
        })
        .await
        .map_err(|e| format!("Đăng nhập cơ quan thuế thất bại: {e}"))?;

    // 3. Chỉ kiểm tra đăng nhập ứng dụng khi bật auto_sync.
    if account.auto_sync {
        // 3. Lấy mật khẩu ứng dụng:
        // Ưu tiên mật khẩu nhập trên form, nếu không có thì đọc SecretStore.
        let app_password = match input.app_password {
            Some(password) if !password.trim().is_empty() => password,
            _ => {
                let key = account
                    .pass_app_key
                    .as_deref()
                    .filter(|key| !key.trim().is_empty())
                    .ok_or_else(|| {
                        "Chưa có mật khẩu ứng dụng. Vui lòng nhập mật khẩu để kiểm tra.".to_string()
                    })?;

                store.get_secret(key).map_err(|e| {
                    format!("Không lấy được mật khẩu ứng dụng. Vui lòng nhập lại mật khẩu: {e}")
                })?
            }
        };
        let type_app = TypeApp::try_from(account.type_app.as_str()).map_err(|e| e.to_string())?;
        // 4. Kiểm tra đăng nhập vaOnline hoặc vaOne.
        auth.check_application_login(account, &app_password, &type_app)
            .await
            .map_err(|e| format!("Đăng nhập ứng dụng thất bại: {e}"))?;
    }
    Ok(token)
}
