use crate::models::agent_info::AgentInfo;
use crate::state::APP_HANDLE;
use crate::state::CURRENT_ROUTE;
use crate::window_config;
use std::sync::Mutex;
use tauri::{LogicalPosition, LogicalSize, Manager, Position, Size};
#[tauri::command]
pub fn set_current_route(route: String) {
    *CURRENT_ROUTE
        .get_or_init(|| Mutex::new(String::new()))
        .lock()
        .unwrap() = route;
}

#[tauri::command]
pub fn quit_app(app: tauri::AppHandle) {
    // crate::state::update_sync_emit(|s| {
    //     s.source = "TCT".to_string();
    //     s.running = false;
    //     s.current_invoice = None;
    // });
    app.exit(0);
}

#[tauri::command]
pub fn get_agent_info() -> AgentInfo {
    AgentInfo {
        name: "vaOne-Plugin".to_string(),
        version: env!("CARGO_PKG_VERSION").into(),
        os: std::env::consts::OS.into(),
    }
}
#[tauri::command]
pub fn page_ready(name: String, show: Option<bool>) {
    if let Some(app) = APP_HANDLE.get() {
        if let Some(window) = app.get_webview_window("main") {
            let show = show.unwrap_or(true);
            let (width, height) = window_config::get_window_size(&name).unwrap();
            if show {
                let _ = window.set_size(Size::Logical(LogicalSize { width, height }));
                // let _ = window.center();
                if let Some(monitor) = window.current_monitor().unwrap() {
                    let scale = monitor.scale_factor();
                    let monitor_size = monitor.size();

                    let monitor_width = monitor_size.width as f64 / scale;
                    let monitor_height = monitor_size.height as f64 / scale;

                    let max_width = monitor_width * 0.90;
                    let max_height = monitor_height * 0.90;

                    let width = width.min(max_width);
                    let height = height.min(max_height);

                    let x = (monitor_width - width) / 2.0;
                    let y = (monitor_height - height) / 2.0;
                    let _ = window.set_position(Position::Logical(LogicalPosition { x, y }));
                }
                let can_resize = matches!(name.as_str(), "report" | "***");
                let _ = window.set_resizable(can_resize);

                let _ = window.show();
                let _ = window.set_focus();
            } else {
                let _ = window.hide();
                let _ = window.set_size(Size::Logical(LogicalSize { width, height }));
            }
        }
    }
}
