mod commands;
mod models;
mod services;
mod state;
mod utils;
mod window_config;
use commands::system::get_agent_info;
use commands::system::get_invoice_detail;
use commands::system::page_ready;
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
            // let info = MenuItem::with_id(app, "agent_info", "Giới thiệu", true, None::<&str>)?;
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

            tauri::async_runtime::spawn(async {
                services::local_server::start().await;
            });

            Ok(())
        })
        // HANDLE MENU CLICK
        .on_menu_event(|app, event| match event.id.as_ref() {
            "agent_info" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                    let _ = window.eval(
                        r#"
                            window.location.hash = "/";
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
                // let _ = app.emit(
                //     "dialog:open",
                //     serde_json::json!({
                //         "id": "quit-1",
                //         "type": "question",
                //         "title": "Thoát ứng dụng",
                //         "message": "Bạn có muốn thoát vaOne-agent không?"
                //     }),
                // );

                app.dialog()
                    .message("Bạn có muốn thoát ứng dụng vaOne-agent không?")
                    .title("Thoát ứng dụng")
                    .buttons(tauri_plugin_dialog::MessageDialogButtons::YesNo)
                    .show(|answer| {
                        if answer {
                            std::process::exit(0);
                        }
                    });
            }

            _ => {}
        })
        // REGISTER COMMAND
        .invoke_handler(tauri::generate_handler![
            get_agent_info,
            get_invoice_detail,
            page_ready
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
