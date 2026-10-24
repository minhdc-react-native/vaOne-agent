use crate::state::{APP_HANDLE, CURRENT_ROUTE};
use std::io::Read;
use tauri::Manager;

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

pub fn decompress_zstd_json<T>(bytes: &[u8]) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let mut decoder = zstd::Decoder::new(bytes).map_err(|e| e.to_string())?;

    let mut json = String::new();

    decoder
        .read_to_string(&mut json)
        .map_err(|e| e.to_string())?;

    serde_json::from_str::<T>(&json).map_err(|e| e.to_string())
}
