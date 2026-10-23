mod api;
mod auth;
mod commands;
mod models;
mod services;
mod state;
mod utils;
mod window_config;
use crate::api::http::wait;
use crate::models::system::AppState;
use crate::services::tct::tax_sync_adapter::{
    ConsoleSyncObserver, LocalCaptchaSolver, MacOsPasswordProvider,
};
use crate::services::update::check_update_on_startup;
use crate::services::worker_manage::TaxSyncWorkerManager;
use crate::state::{APP_STATE, SYNC_MENU};
use crate::state::{CURRENT_ROUTE, ONLINE_MENU};
use crate::utils::public::navigate_to_route;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    App, Manager,
};
use vaone_tax_sync::database::Database;
use vaone_tax_sync::worker::TaxSyncWorkerConfig;
use vaone_tax_sync::TaxSyncWorker;
const LINE_TEXT: &str = "------------------------------";

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
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {
            println!("Application is already running.");
        }))
        // INIT APP
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                let pdfium_path = get_pdfium_path(&app)?;

                println!("PDFium: {}", pdfium_path.display());
                println!("Exists: {}", pdfium_path.exists());
                printer_core::init_pdfium(pdfium_path);
            }
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            let _ = window_clone.hide();
            let online = MenuItem::with_id(
                app,
                "online",
                "○ Chưa kết nối nguồn hóa đơn",
                true,
                None::<&str>,
            )?;
            let _ = ONLINE_MENU.set(online.clone());

            let sync = MenuItem::with_id(app, "sync", LINE_TEXT, false, None::<&str>)?;

            let _ = SYNC_MENU.set(sync.clone());

            let check_update =
                MenuItem::with_id(app, "update", "Kiểm tra phiên bản", true, None::<&str>)?;

            // Menu Tổng cục thuế
            let tax_menu = MenuItem::with_id(
                app,
                "tax_config",
                "Cấu hình kết nối tổng cục thuế",
                true,
                None::<&str>,
            )?;

            let settings = MenuItem::with_id(app, "settings", "Cài đặt", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit = MenuItem::with_id(app, "quit", "Thoát ứng dụng", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &online,
                    &sync,
                    &tax_menu,
                    // &separator,
                    &settings,
                    &check_update,
                    &separator,
                    &quit,
                ],
            )?;

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

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                wait(Some(3000)).await;
                if let Some(splash) = app_handle.get_webview_window("splash_screen") {
                    let _ = splash.close();
                    check_update_on_startup(app_handle, Some(true));
                }
            });

            // khởi tạo dữ liệu SQLite
            let app_data_dir = app.path().app_data_dir()?;

            std::fs::create_dir_all(&app_data_dir)?;

            let db_path = app_data_dir.join("tax-sync.sqlite3");
            println!("Tax sync database: {}", db_path.display());
            let db = Database::new(db_path).map_err(|e| std::io::Error::other(e.to_string()))?;

            app.manage(db.clone());

            let captcha_solver = Arc::new(LocalCaptchaSolver);

            let sync_observer = Arc::new(ConsoleSyncObserver);

            let worker = Arc::new(TaxSyncWorker::new(
                db.clone(),
                captcha_solver,
                sync_observer,
                TaxSyncWorkerConfig::new(),
            )?);

            let worker_manager = TaxSyncWorkerManager::new(worker);

            // Tự chạy khi ứng dụng khởi động
            worker_manager.start().map_err(std::io::Error::other)?;

            app.manage(worker_manager);

            Ok(())
        })
        // HANDLE MENU CLICK
        .on_menu_event(|app, event| match event.id.as_ref() {
            "online" => {
                let _ = navigate_to_route("/login");
            }
            "sync" => {
                let _ = navigate_to_route("/get-invoices");
            }
            "update" => {
                let _ = check_update_on_startup(app.clone(), Some(false));
            }
            "report" => {
                let _ = navigate_to_route("/report");
            }
            "tax_config" => {
                let _ = navigate_to_route("/tax-config");
            }
            "settings" => {
                let _ = navigate_to_route("/settings");
            }
            "quit" => {
                let _ = navigate_to_route("/quit");
            }
            _ => {}
        })
        // REGISTER COMMAND
        .invoke_handler(tauri::generate_handler![
            commands::system::quit_app,
            commands::system::get_agent_info,
            commands::system::set_current_route,
            commands::system::connect_invoice,
            commands::api_command::http_get,
            commands::api_command::http_post,
            commands::system::page_ready,
            commands::system::call_emit_event,
            commands::pdf::render_pdf,
            commands::invoice::get_sync_state,
            commands::invoice::start_invoice_tct_sync,
            commands::invoice::start_save_invoice_sync,
            commands::invoice::start_m_invoice_sync,
            commands::system::captcha_train,
            commands::system::captcha_predict,
            commands::tax_accounts::get_tax_accounts,
            commands::tax_accounts::create_tax_account,
            commands::tax_accounts::update_tax_account,
            commands::tax_accounts::update_auto_sync,
            commands::tax_accounts::delete_tax_account,
            commands::tax_accounts::check_tax_application_login,
            commands::tax_accounts::check_app_login,
            commands::sync_invoices::get_invoice_stats,
            commands::sync_invoices::get_sync_days,
            commands::sync_invoices::get_sync_invoices,
            commands::sync_invoices::reset_sync_status,
            commands::sync_invoices::start_tax_sync_worker,
            commands::sync_invoices::stop_tax_sync_worker,
            commands::sync_invoices::is_tax_sync_worker_running
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn progress_bar(done: Option<usize>, total: Option<usize>) {
    if let Some(item) = SYNC_MENU.get() {
        match (done, total) {
            (Some(done), Some(total)) => {
                let width = 10;
                let filled = if total == 0 { 0 } else { done * width / total };
                let percent = if total == 0 { 0 } else { done * 100 / total };
                let text = format!(
                    "[{}{}] {}/{} ({}%)",
                    "■".repeat(filled),
                    "□".repeat(width - filled),
                    done,
                    total,
                    percent
                );
                // let text = format!("Tải hóa đơn: {}/{} ({}%)", done, total, percent);
                let _ = item.set_enabled(true);
                let _ = item.set_text(&text);
            }
            _ => {
                let _ = item.set_enabled(false);
                let _ = item.set_text(LINE_TEXT);
            }
        }
    }
}

fn get_pdfium_path(app: &App) -> tauri::Result<PathBuf> {
    #[cfg(debug_assertions)]
    {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources");
        path.push("pdfium");
        path.push("pdfium.dll");
        Ok(path)
    }

    #[cfg(not(debug_assertions))]
    {
        app.path().resolve(
            "resources/pdfium/pdfium.dll",
            tauri::path::BaseDirectory::Resource,
        )
    }
}
