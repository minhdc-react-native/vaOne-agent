use tauri::Manager;

use crate::{
    models::system::TokenState,
    state::{APP_HANDLE, APP_STATE, CURRENT_ROUTE},
};

pub fn navigate_to_route(route: &str) {
    if let Some(app) = APP_HANDLE.get() {
        if let Some(window) = app.get_webview_window("main") {
            let current = CURRENT_ROUTE.get().unwrap().lock().unwrap().clone();
            if current == route {
                let _ = window.set_focus();
            } else {
                let _ = window.hide();
                match window.eval(&format!(
                    r#"
                        window.location.hash = "{}";
                    "#,
                    route
                )) {
                    Ok(_) => println!("eval ok"),
                    Err(e) => println!("eval err: {:?}", e),
                }
            }
        }
    }
}
