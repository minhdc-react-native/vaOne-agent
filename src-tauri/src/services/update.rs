use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

use tauri_plugin_dialog::MessageDialogButtons;

use crate::{models::system::UpdateAvailablePayload, state::APP_HANDLE};

pub fn check_update_on_startup(app: AppHandle, silent: Option<bool>) {
    println!("[UPDATE] Function called");
    let silent = silent.unwrap_or(false);
    tauri::async_runtime::spawn(async move {
        println!("[UPDATE] Task started");
        let Ok(updater) = app.updater() else {
            println!("[UPDATE] Cannot create updater");
            return;
        };
        println!("[UPDATE] Checking...");
        match updater.check().await {
            Ok(Some(update)) => {
                let current_version = app.package_info().version.to_string();
                let notes = update
                    .body
                    .as_deref()
                    .unwrap_or("Không có ghi chú cập nhật.");

                app.dialog()
                    .message(format!(
                        "Đã có phiên bản mới.\n\n\
                        Phiên bản hiện tại: {}\n\
                        Phiên bản mới: {}\n\n\
                        Nội dung cập nhật:\n{}\n\n\
                        Bạn có muốn cập nhật ngay không?",
                        current_version, update.version, notes
                    ))
                    .title("vaOne-Plugin")
                    .kind(MessageDialogKind::Info)
                    .buttons(MessageDialogButtons::OkCancel)
                    .show(move |ok| {
                        if !ok {
                            return;
                        }

                        tauri::async_runtime::spawn(async move {
                            if let Err(err) = update_and_restart(app.clone(), current_version).await
                            {
                                eprintln!("Update failed: {}", err);
                            }
                        });
                    });
            }

            Ok(None) => {
                if !silent {
                    let current_version = app.package_info().version.to_string();
                    app.dialog()
                        .message(format!(
                            "Bạn đang sử dụng phiên bản mới nhất: {}.",
                            current_version
                        ))
                        .title("vaOne-Plugin")
                        .show(|_| {});
                }
            }

            Err(err) => {
                if !silent {
                    app.dialog()
                        .message(format!("Không thể kiểm tra cập nhật.\n\n{}", err))
                        .title("vaOne-Plugin")
                        .show(|_| {});
                }
            }
        }
        println!("[UPDATE] Check completed");
    });
}

// pub fn check_update_on_startup(app: AppHandle, silent: Option<bool>) {
//     let silent = silent.unwrap_or(false);

//     tauri::async_runtime::spawn(async move {
//         println!("[UPDATE] Checking...");

//         let Ok(updater) = app.updater() else {
//             eprintln!("[UPDATE] Cannot create updater");
//             return;
//         };

//         match updater.check().await {
//             Ok(Some(update)) => {
//                 let payload = UpdateAvailablePayload {
//                     current_version: app.package_info().version.to_string(),
//                     new_version: update.version.to_string(),
//                     notes: update
//                         .body
//                         .clone()
//                         .unwrap_or_else(|| "Không có ghi chú cập nhật.".to_string()),
//                 };

//                 if let Err(err) = app.emit("update-available", payload) {
//                     eprintln!("[UPDATE] Emit failed: {err}");
//                 }
//             }

//             Ok(None) => {
//                 if !silent {
//                     let version = app.package_info().version.to_string();

//                     let _ = app.emit("update-not-available", version);
//                 }
//             }

//             Err(err) => {
//                 eprintln!("[UPDATE] Check failed: {err}");

//                 if !silent {
//                     let _ = app.emit("update-check-failed", err.to_string());
//                 }
//             }
//         }
//     });
// }

fn emit_progress(progress: serde_json::Value) {
    if let Some(app) = APP_HANDLE.get() {
        let _ = app.emit("update-progress", progress);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

pub async fn update_and_restart(app: AppHandle, current_version: String) -> anyhow::Result<()> {
    let Some(update) = app.updater()?.check().await? else {
        println!("Already latest.");
        return Ok(());
    };
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();

        let _ = window.eval(
            r#"
            window.location.hash = "/update";
            "#,
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    println!("Downloading version {}", update.version);

    emit_progress(json!({
        "currentVersion":current_version,
        "newVersion": update.version
    }));

    let mut downloaded = 0;

    let bytes = update
        .download(
            |chunk_length, content_length| {
                downloaded += chunk_length;

                let downloaded_mb = ((downloaded as f64 / 1024.0 / 1024.0) * 100.0).round() / 100.0;

                let total_mb = content_length
                    .map(|v| ((v as f64 / 1024.0 / 1024.0) * 100.0).round() / 100.0)
                    .unwrap_or(0.0);

                emit_progress(json!({
                    "message":"Tải dữ liệu...",
                    "downloaded": downloaded_mb,
                    "total": total_mb,
                }));

                println!("Downloaded {} / {:?}", downloaded_mb, total_mb);
            },
            || {
                println!("Download completed.");
            },
        )
        .await?;

    println!("Installing...");

    emit_progress(json!({
        "message":"Đang cài đặt...",
        "downloaded": 0,
    }));

    update.install(bytes)?;

    println!("Restarting...");
    emit_progress(json!({
        "message":"Khởi động lại...",
        "finish": true,
    }));

    app.restart();

    Ok(())
}

pub fn update_progress(payload: Option<&Value>) {
    if let Some(app) = APP_HANDLE.get() {
        let _ = app.emit("sync-progress", payload);
    }
}
