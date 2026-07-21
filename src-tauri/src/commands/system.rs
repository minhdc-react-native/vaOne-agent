use crate::models::system::AgentInfo;
use crate::state::APP_HANDLE;
use crate::state::CURRENT_ROUTE;
use crate::state::ONLINE_MENU;
use crate::window_config;
use captcha_db::DATABASE_NAME;
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
pub fn connect_invoice(new_label: String) -> Result<(), String> {
    if let Some(item) = ONLINE_MENU.get() {
        item.set_text(&format!("● {}", new_label))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
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
            let (mut width, mut height) = window_config::get_window_size(&name).unwrap();
            if show {
                if let Some(monitor) = window.current_monitor().unwrap() {
                    let scale = monitor.scale_factor();
                    let monitor_size = monitor.size();

                    let monitor_width = monitor_size.width as f64 / scale;
                    let monitor_height = monitor_size.height as f64 / scale;

                    width = width.min(monitor_width * 0.9);
                    height = height.min(monitor_height * 0.9);

                    let _ = window.set_size(Size::Logical(LogicalSize { width, height }));

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

#[tauri::command]
pub fn captcha_train(svg: String, answer: String) -> Result<(), String> {
    let path = get_path(DATABASE_NAME).map_err(|e| e.to_string())?;

    // if std::path::Path::new(path).exists() {
    //     std::fs::remove_file(path).map_err(|e| e.to_string())?;
    // }

    let mut trainer = captcha_db::Trainer::load(path.clone()).map_err(|e| e.to_string())?;

    trainer.train(&svg, &answer).map_err(|e| e.to_string())?;

    trainer.save(path).map_err(|e| e.to_string())?;

    Ok(())
}

use captcha_db::{svg::parse_svg, Database, Matcher};

#[tauri::command]
pub fn captcha_predict(svg: String) -> Result<String, String> {
    let path = get_path(DATABASE_NAME).map_err(|e| e.to_string())?;

    // Load database
    let db = Database::load(path).map_err(|e| e.to_string())?;

    // Parse SVG
    let glyphs = parse_svg(&svg).map_err(|e| e.to_string())?;

    // Match
    let matcher = Matcher::new(db);

    let result = matcher.recognize(&glyphs);

    Ok(result)
}

fn get_path(db_name: &str) -> Result<std::path::PathBuf, String> {
    let app = APP_HANDLE
        .get()
        .ok_or_else(|| "AppHandle chưa được khởi tạo".to_string())?;

    let mut path = app.path().app_data_dir().map_err(|e| e.to_string())?;

    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;

    path.push(db_name);

    Ok(path)
}
