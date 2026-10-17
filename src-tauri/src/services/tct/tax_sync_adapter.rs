use anyhow::{Context, Result};
use keyring::Entry;

use vaone_tax_sync::worker::{CaptchaSolver, PasswordProvider};

pub struct MacOsPasswordProvider;

impl PasswordProvider for MacOsPasswordProvider {
    fn get_password(&self, password_key: &str) -> Result<String> {
        let entry = Entry::new("vaOne-agent.tax-sync", password_key)
            .context("Không tạo được Keychain entry")?;

        entry
            .get_password()
            .context("Không đọc được mật khẩu từ macOS Keychain")
    }
}

pub struct LocalCaptchaSolver;

impl CaptchaSolver for LocalCaptchaSolver {
    fn solve(&self, svg: &str) -> Result<String, String> {
        captcha_predict(svg.to_string())
    }
}

use vaone_tax_sync::models::{InvoiceProgress, SyncSummary};
use vaone_tax_sync::sync::SyncObserver;

use crate::commands::system::captcha_predict;

pub struct ConsoleSyncObserver;

impl SyncObserver for ConsoleSyncObserver {
    fn on_started(&self, total: usize) {
        println!("[tax-sync] Bắt đầu xử lý {total} hóa đơn");
    }

    fn on_invoice(&self, progress: InvoiceProgress) {
        if progress.success {
            println!(
                "[tax-sync] {}/{} - Hóa đơn {} thành công",
                progress.completed, progress.total, progress.invoice_number
            );
        } else {
            eprintln!(
                "[tax-sync] {}/{} - Hóa đơn {} lỗi: {}",
                progress.completed,
                progress.total,
                progress.invoice_number,
                progress.error.as_deref().unwrap_or("Không rõ lỗi")
            );
        }
    }

    fn on_finished(&self, summary: &SyncSummary) {
        println!(
            "[tax-sync] Hoàn tất: tổng={}, thành công={}, lỗi={}, đã hủy={}",
            summary.total, summary.success, summary.failed, summary.cancelled
        );
    }
}
