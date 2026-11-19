use crate::state::APP_HANDLE;
use tauri_plugin_notification::NotificationExt;

pub fn show(id: i32, title: &str, message: &str) -> Result<(), String> {
    let app = APP_HANDLE
        .get()
        .ok_or_else(|| "AppHandle not initialized".to_string())?;

    app.notification()
        .builder()
        .id(id)
        .title(title)
        .body(message)
        .show()
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationType {
    Normal,
    TaxConfig,
}

impl NotificationType {
    pub fn id(self) -> i32 {
        match self {
            Self::Normal => 1000,
            Self::TaxConfig => 1001,
        }
    }

    pub fn route(self) -> Option<&'static str> {
        match self {
            Self::Normal => None,
            Self::TaxConfig => Some("/tax-config"),
        }
    }
}
