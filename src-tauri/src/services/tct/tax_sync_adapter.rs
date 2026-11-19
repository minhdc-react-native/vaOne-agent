use tauri::Emitter;

use vaone_tax_sync::models::{InvoiceTask, TaskProgress, TaskStatus, TaxAccount};
use vaone_tax_sync::sync::SyncObserver;
use vaone_tax_sync::worker::CaptchaSolver;

use crate::commands::system::captcha_predict;
use crate::services::progress_store::ProgressStore;
use crate::state::APP_HANDLE;
use crate::utils::notification::{show, NotificationType};
use tauri::Manager;

pub struct LocalCaptchaSolver;

impl CaptchaSolver for LocalCaptchaSolver {
    fn solve(&self, svg: &str) -> Result<String, String> {
        captcha_predict(svg.to_string())
    }
}
pub struct ConsoleSyncObserver;

impl SyncObserver for ConsoleSyncObserver {
    fn on_progress(&self, progress: TaskProgress) {
        let task_name = progress.task.label();

        let status = match progress.status {
            TaskStatus::Running => "Đang chạy",
            TaskStatus::Completed => "Hoàn thành",
            TaskStatus::CompletedWithErrors => "Hoàn thành có lỗi",
            TaskStatus::Failed => "Thất bại",
            TaskStatus::Cancelled => "Đã hủy",
        };

        println!(
            "[tax-sync][{}][{}][{}] {}/{:?} - {}",
            progress.tax_code,
            task_name,
            status,
            progress.completed,
            progress.total,
            progress.message,
        );

        if let Some(error) = &progress.error {
            eprintln!(
                "[tax-sync][{}][{}] Lỗi: {}",
                progress.tax_code, task_name, error
            );
        }

        if let Some(app) = APP_HANDLE.get() {
            // Cập nhật trạng thái hiện tại.
            app.state::<ProgressStore>().update(progress.clone());

            // Gửi event realtime.
            if let Err(error) = app.emit("tax-sync-progress", &progress) {
                eprintln!("[tax-sync] Không gửi được progress event: {error}");
            }
        }
    }

    fn on_notification(&self, tax_accounts: &Vec<TaxAccount>) {
        let count = tax_accounts.len();

        if count == 0 {
            return;
        }

        let tax_codes = tax_accounts
            .iter()
            .take(3)
            .map(|account| account.tax_code.as_str())
            .collect::<Vec<_>>();

        let mut message = format!(
            "Có {} tài khoản thuế cần kiểm tra:\n{}",
            count,
            tax_codes.join("\n")
        );

        if count > 3 {
            message.push_str(&format!("\n... và {} mã số thuế khác", count - 3));
        }

        match show(NotificationType::TaxConfig.id(), "vaOne Agent", &message) {
            Ok(()) => println!("[notification] Gửi thành công"),
            Err(error) => {
                eprintln!("[notification] Gửi thất bại: {error}");
            }
        }
    }
}
