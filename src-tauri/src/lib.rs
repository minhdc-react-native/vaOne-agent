mod api;
mod commands;
mod models;
pub mod pdf;
mod services;
mod state;
mod utils;
mod window_config;
use crate::state::CURRENT_ROUTE;
use crate::state::{AppState, APP_STATE};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        // INIT APP
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            let _ = window_clone.hide();
            // let report = MenuItem::with_id(app, "report", "Test print pdf", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Cài đặt", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "Thoát ứng dụng", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&settings, &separator, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("vaOne plugin")
                .build(app)?;

            crate::state::APP_HANDLE
                .set(app.handle().clone())
                .expect("APP_HANDLE already initialized");

            CURRENT_ROUTE.set(Mutex::new(String::new())).ok();

            APP_STATE
                .set(std::sync::Mutex::new(AppState::default()))
                .unwrap();

            crate::state::init_ws_state();

            tauri::async_runtime::spawn(async {
                services::local_server::start().await;
            });

            Ok(())
        })
        // HANDLE MENU CLICK
        .on_menu_event(|app, event| match event.id.as_ref() {
            "report" => {
                if let Some(window) = app.get_webview_window("main") {
                    let current = CURRENT_ROUTE.get().unwrap().lock().unwrap().clone();
                    if current == "/report" {
                        let _ = window.set_focus();
                    } else {
                        let _ = window.hide();
                        let _ = window.eval(
                            r#"
                            window.location.hash = "/report";
                        "#,
                        );
                    }
                }
            }

            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let current = CURRENT_ROUTE.get().unwrap().lock().unwrap().clone();
                    if current == "/settings" {
                        let _ = window.set_focus();
                    } else {
                        let _ = window.hide();
                        let _ = window.eval(
                            r#"
                            window.location.hash = "/settings";
                        "#,
                        );
                    }
                }
            }
            "quit" => {
                if let Some(window) = app.get_webview_window("main") {
                    let current = CURRENT_ROUTE.get().unwrap().lock().unwrap().clone();
                    if current == "/quit" {
                        let _ = window.set_focus();
                    } else {
                        let _ = window.hide();
                        let _ = window.eval(
                            r#"
                            window.location.hash = "/quit";
                        "#,
                        );
                    }
                }
            }

            _ => {}
        })
        // REGISTER COMMAND
        .invoke_handler(tauri::generate_handler![
            commands::system::quit_app,
            commands::system::get_agent_info,
            commands::system::set_current_route,
            commands::api_command::http_get,
            commands::api_command::http_post,
            commands::system::page_ready,
            commands::pdf::render_pdf,
            commands::printer::get_printer_list,
            commands::printer::print_pdf,
            commands::invoice::get_sync_state,
            commands::invoice::start_invoice_tct_sync
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
