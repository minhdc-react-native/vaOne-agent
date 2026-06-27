use tauri_plugin_notification::NotificationExt;

use crate::state::APP_HANDLE;

pub fn show(title: &str, message: &str) -> Result<(), String> {
    let app = APP_HANDLE
        .get()
        .ok_or_else(|| "AppHandle not initialized".to_string())?;

    app.notification()
        .builder()
        .title(title)
        .body(message)
        .show()
        .map_err(|e| e.to_string())
}
