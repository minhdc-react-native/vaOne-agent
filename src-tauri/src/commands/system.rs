use crate::models::system::AgentInfo;
use crate::models::system::Login;
use crate::state::APP_HANDLE;
use crate::state::APP_STATE;
use crate::state::CURRENT_ROUTE;
use crate::state::ONLINE_MENU;
use crate::state::WS_STATE;
use crate::window_config;
use captcha_db::DATABASE_NAME;
use serde::Serialize;
use serde_json::Value;
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
pub fn call_emit_event(event: String, tenant_id: String, data: Value) -> Result<(), String> {
    if let Some(ws) = WS_STATE.get() {
        ws.broadcast_json(&event, &tenant_id, &data);
    }

    Ok(())
}

#[tauri::command]
pub fn connect_invoice(login: Login) -> Result<(), String> {
    let new_label = format!("{}:{}", login.source, login.username);

    if let Some(item) = ONLINE_MENU.get() {
        item.set_text(&format!("● {}", new_label))
            .map_err(|e| e.to_string())?;
    }

    let app_state = APP_STATE.get().ok_or("APP_STATE chưa được khởi tạo")?;

    let mut state = app_state.lock().map_err(|e| e.to_string())?;

    let tenant = state.tenants.entry(login.tenant_id.clone()).or_default();

    tenant.info_login = Some(login);

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
pub fn page_ready(name: String, show: Option<bool>, full_screen: Option<bool>) {
    if let Some(app) = APP_HANDLE.get() {
        if let Some(window) = app.get_webview_window("main") {
            let show = show.unwrap_or(true);
            let full_screen = full_screen.unwrap_or(false);

            let (mut width, mut height) = window_config::get_window_size(&name).unwrap();

            if show {
                let can_resize = matches!(name.as_str(), "report" | "historyInvoice");
                let _ = window.set_resizable(can_resize);

                if let Ok(Some(monitor)) = window.current_monitor() {
                    let scale = monitor.scale_factor();
                    let work_area = monitor.work_area();

                    let work_x = work_area.position.x as f64 / scale;
                    let work_y = work_area.position.y as f64 / scale;
                    let work_width = work_area.size.width as f64 / scale;
                    let work_height = work_area.size.height as f64 / scale;

                    if full_screen {
                        // Chiếm toàn bộ work_area, không bật fullscreen hệ thống.
                        width = work_width;
                        height = work_height;

                        let _ = window.set_size(Size::Logical(LogicalSize { width, height }));

                        let _ = window.set_position(Position::Logical(LogicalPosition {
                            x: work_x,
                            y: work_y,
                        }));
                    } else {
                        // Thoát fullscreen hệ thống nếu cửa sổ trước đó đang dùng.
                        let _ = window.set_fullscreen(false);

                        width = width.min(work_width * 0.9);
                        height = height.min(work_height * 0.9);

                        let _ = window.set_size(Size::Logical(LogicalSize { width, height }));

                        let x = work_x + (work_width - width) / 2.0;
                        let y = work_y + (work_height - height) / 2.0;

                        let _ = window.set_position(Position::Logical(LogicalPosition { x, y }));
                    }
                }

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
    predict_captcha(&svg)
}

pub fn predict_captcha(svg: &str) -> Result<String, String> {
    let path = get_path(DATABASE_NAME).map_err(|e| e.to_string())?;
    let db = Database::load(path).map_err(|e| e.to_string())?;
    let glyphs = parse_svg(svg).map_err(|e| e.to_string())?;
    let matcher = Matcher::new(db);

    Ok(matcher.recognize(&glyphs))
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
