use tauri::Manager;

use crate::state::{APP_HANDLE, CURRENT_ROUTE};

pub fn navigate_to_route(route: &str) {
    if let Some(app) = APP_HANDLE.get() {
        if let Some(window) = app.get_webview_window("main") {
            let current = CURRENT_ROUTE.get().unwrap().lock().unwrap().clone();
            if current == route {
                let _ = window.set_focus();
            } else {
                let _ = window.hide();
                let _ = window.eval(&format!(
                    r#"
                window.location.hash = "{}";
            "#,
                    route
                ));
            }
        }
    }
}
