mod api;
mod commands;
mod models;
pub mod pdf;
mod services;
mod state;
mod utils;
mod window_config;
use crate::state::{AppState, APP_STATE};

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_dialog::DialogExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            let report = MenuItem::with_id(app, "report", "Test print pdf", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "Cài đặt", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "Thoát ứng dụng", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&report, &settings, &separator, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("vaOne plugin")
                .build(app)?;

            crate::state::APP_HANDLE
                .set(app.handle().clone())
                .expect("APP_HANDLE already initialized");

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
                // state::update_sync_emit(|s| {
                //     s.source = "TCT".to_string();
                //     s.running = true;
                //     s.total = 100;
                //     s.completed = 0;
                //     s.current_invoice = Some(json!({
                //         "invoiceNumber": "112345",
                //         "b": 123,
                //         "c": true
                //     }));
                // });

                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                    let _ = window.eval(
                        r#"
                            window.location.hash = "/report";
                        "#,
                    );
                }
            }

            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                    let _ = window.eval(
                        r#"
                            window.location.hash = "/settings";
                        "#,
                    );
                }
            }
            "quit" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                    let _ = window.eval(
                        r#"
                            window.location.hash = "/quit";
                        "#,
                    );
                }
                // let _ = app.emit(
                //     "dialog:open",
                //     serde_json::json!({
                //         "id": "quit-1",
                //         "type": "question",
                //         "title": "Thoát ứng dụng",
                //         "message": "Bạn có muốn thoát vaOne-agent không?"
                //     }),
                // );

                // app.dialog()
                //     .message("Bạn có muốn thoát ứng dụng vaOne-agent không?")
                //     .title("Thoát ứng dụng")
                //     .buttons(tauri_plugin_dialog::MessageDialogButtons::YesNo)
                //     .show(|answer| {
                //         if answer {
                //             std::process::exit(0);
                //         }
                //     });
            }

            _ => {}
        })
        // REGISTER COMMAND
        .invoke_handler(tauri::generate_handler![
            commands::system::quit_app,
            commands::system::get_agent_info,
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
