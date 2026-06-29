mod commands;
mod models;
mod services;
mod state;
mod utils;
mod window_config;
use commands::system::get_agent_info;
use commands::system::get_invoice_detail;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, LogicalSize, Manager, Size, WebviewWindow, WindowEvent,
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
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window_clone.emit("app-hide", ());
                    let _ = window_clone.hide();
                }
            });

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
                    let (width, height) = window_config::get_window_size("splash").unwrap();
                    let _ = window.set_size(Size::Logical(LogicalSize { width, height }));
                    let _ = window.eval(
                        r#"
                            window.location.hash = "/";
                        "#,
                    );

                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        let _ = window.center();
                        let _ = window.show();
                        let _ = window.set_focus();
                    });
                }
                // let info = get_agent_info();

                // let msg = format!("{}\nVersion: {}\nOS: {}", info.name, info.version, info.os);

                // let _ = app
                //     .notification()
                //     .builder()
                //     .title("vaOne Info")
                //     .body(msg)
                //     .show();
            }

            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let (width, height) = window_config::get_window_size("settings").unwrap();
                    let _ = window.set_size(Size::Logical(LogicalSize { width, height }));
                    let _ = window.set_title("Cài đặt");
                    let _ = window.eval(
                        r#"
                            window.location.hash = "/settings";
                            window.dispatchEvent(new Event("settings-open"));
                        "#,
                    );

                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        let _ = window.center();
                        let _ = window.show();
                        let _ = window.set_focus();
                    });
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
        .invoke_handler(tauri::generate_handler![get_agent_info, get_invoice_detail])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
