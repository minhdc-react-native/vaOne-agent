use crate::db::invoice_database::{invoice_date_vietnam, invoice_key, InvoiceDatabase};
use crate::error::TaxSyncError;
use crate::global_request_queue::GlobalRequestQueue;
use crate::models::{InvoiceTask, TaskProgress, TaskStatus, TaxAccount};
use crate::worker::{vietnam_today, CaptchaSolver};
use crate::{
    client::TaxClient,
    database::Database,
    models::{InvoiceProgress, SyncRequest, SyncSummary},
};

use anyhow::{anyhow, Result};

use chrono::NaiveDate;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
pub trait SyncObserver: Send + Sync {
    fn on_progress(&self, progress: TaskProgress);
    fn on_notification(&self, tax_accounts: &Vec<TaxAccount>);
}

pub struct TaxSyncService {
    client: TaxClient,
    upload_notify: Arc<Notify>,
    update_notify: Arc<Notify>,
    captcha_solver: Arc<dyn CaptchaSolver>,
    cancel: CancellationToken,
}

impl TaxSyncService {
    pub fn new(
        tax_code: String,
        upload_notify: Arc<Notify>,
        update_notify: Arc<Notify>,
        captcha_solver: Arc<dyn CaptchaSolver>,
        request_queue: Arc<GlobalRequestQueue>,
        cancel: CancellationToken,
    ) -> Result<Self> {
        Ok(Self {
            client: TaxClient::new(tax_code, request_queue.clone(), cancel.clone())?,
            upload_notify,
            update_notify,
            captcha_solver,
            cancel,
        })
    }

    pub async fn run(
        &self,
        invoice_db: &InvoiceDatabase,
        request: SyncRequest,
        observer: &dyn SyncObserver,
    ) -> Result<SyncSummary> {
        if request.tax_code.trim().is_empty() {
            return Err(
                TaxSyncError::InvalidConfig("tax_code must not be empty".to_string()).into(),
            );
        }

        if request.token.trim().is_empty() {
            return Err(TaxSyncError::InvalidConfig("token must not be empty".to_string()).into());
        }

        if self.cancel.is_cancelled() {
            return Ok(SyncSummary {
                cancelled: true,
                ..Default::default()
            });
        }

        let dates = dates_between(&request.from_date, &request.to_date)?;

        // Bước 1: Thông báo bắt đầu lấy danh sách hóa đơn.
        report_download_progress(
            observer,
            &request.tax_code,
            request.invoice_type,
            TaskStatus::Running,
            0,
            None,
            "Đang lấy danh sách hóa đơn từ cơ quan thuế",
            None,
            None,
            None,
        );

        let solver = Arc::clone(&self.captcha_solver);

        let invoices = match self
            .client
            .fetch_all(
                &request.tax_code,
                request.invoice_type,
                &request.from_date,
                &request.to_date,
                &request.token,
                invoice_db,
                move |captcha_content| {
                    let solver = Arc::clone(&solver);
                    let svg = captcha_content.to_string();

                    async move {
                        solver
                            .solve(&svg)
                            .map_err(TaxSyncError::Authentication)
                            .map_err(anyhow::Error::from)
                    }
                },
                |progress| {
                    report_download_progress(
                        observer,
                        &request.tax_code,
                        request.invoice_type,
                        TaskStatus::Running,
                        progress.completed_months,
                        Some(progress.total_months),
                        format!(
                            "{} | Trang {} | Đã nhận {} hóa đơn",
                            progress.message, progress.page, progress.total_invoices,
                        ),
                        None,
                        None,
                        None,
                    );
                },
            )
            .await
        {
            Ok(invoices) => invoices,

            Err(err) => {
                if self.cancel.is_cancelled() {
                    report_download_progress(
                        observer,
                        &request.tax_code,
                        request.invoice_type,
                        TaskStatus::Cancelled,
                        0,
                        None,
                        "Đã hủy quá trình lấy danh sách hóa đơn",
                        None,
                        None,
                        None,
                    );

                    return Ok(SyncSummary {
                        cancelled: true,
                        ..Default::default()
                    });
                }

                let message = err.to_string();

                for date in &dates {
                    let _ = invoice_db.set_day_list_failed(
                        &request.tax_code,
                        request.invoice_type,
                        date,
                        &message,
                    );
                }

                report_download_progress(
                    observer,
                    &request.tax_code,
                    request.invoice_type,
                    TaskStatus::Failed,
                    0,
                    None,
                    "Không thể lấy danh sách hóa đơn",
                    None,
                    Some(false),
                    Some(message),
                );

                return Err(err);
            }
        };

        if self.cancel.is_cancelled() {
            report_download_progress(
                observer,
                &request.tax_code,
                request.invoice_type,
                TaskStatus::Cancelled,
                0,
                Some(invoices.len()),
                "Đã hủy quá trình tải hóa đơn",
                None,
                None,
                None,
            );

            return Ok(SyncSummary {
                cancelled: true,
                ..Default::default()
            });
        }

        // Bước 2: Đã lấy được danh sách, bắt đầu lưu SQLite.
        let total = invoices.len();

        let mut summary = SyncSummary {
            total,
            ..Default::default()
        };

        report_download_progress(
            observer,
            &request.tax_code,
            request.invoice_type,
            TaskStatus::Running,
            0,
            Some(total),
            format!("Đã lấy được {total} hóa đơn. Đang chuẩn bị lưu vào SQLite"),
            None,
            None,
            None,
        );

        let today = vietnam_today().await?;
        let today_string = today.format("%Y-%m-%d").to_string();

        let mut processed = 0usize;

        for (index, item) in invoices.iter().enumerate() {
            if self.cancel.is_cancelled() {
                summary.cancelled = true;
                break;
            }

            // Lấy khóa hóa đơn.
            let invoice_key = match invoice_key(item) {
                Ok(key) => key,
                Err(err) => {
                    summary.failed += 1;
                    processed += 1;

                    report_download_progress(
                        observer,
                        &request.tax_code,
                        request.invoice_type,
                        TaskStatus::Running,
                        processed,
                        Some(total),
                        "Không thể xác định khóa hóa đơn",
                        Some(item),
                        Some(false),
                        Some(err.to_string()),
                    );

                    continue;
                }
            };

            // Kiểm tra hóa đơn đã tồn tại hay chưa.
            let exists = invoice_db.invoice_today_exists(
                &request.tax_code,
                request.invoice_type,
                &invoice_key,
            )?;

            if exists {
                processed += 1;

                report_download_progress(
                    observer,
                    &request.tax_code,
                    request.invoice_type,
                    TaskStatus::Running,
                    processed,
                    Some(total),
                    "Hóa đơn đã tồn tại, bỏ qua",
                    Some(item),
                    Some(true),
                    None,
                );

                continue;
            }

            if self.cancel.is_cancelled() {
                summary.cancelled = true;
                break;
            }

            // Lưu summary vào SQLite.
            let result = (|| -> Result<()> {
                let date = invoice_date_vietnam(item)?;

                let is_today = date == today_string;

                invoice_db.save_invoice_summary(
                    &request.tax_code,
                    request.invoice_type,
                    item,
                    is_today,
                )?;

                Ok(())
            })();

            processed += 1;

            match result {
                Ok(()) => {
                    summary.saved += 1;

                    self.update_notify.notify_one();

                    report_download_progress(
                        observer,
                        &request.tax_code,
                        request.invoice_type,
                        TaskStatus::Running,
                        processed,
                        Some(total),
                        "Đã lưu hóa đơn vào SQLite",
                        Some(item),
                        Some(true),
                        None,
                    );
                }

                Err(err) => {
                    summary.failed += 1;

                    report_download_progress(
                        observer,
                        &request.tax_code,
                        request.invoice_type,
                        TaskStatus::Running,
                        processed,
                        Some(total),
                        "Lỗi khi lưu hóa đơn vào SQLite",
                        Some(item),
                        Some(false),
                        Some(err.to_string()),
                    );
                }
            }
        }

        // Bước 3: Kiểm tra trạng thái sau khi lưu.
        if summary.cancelled || self.cancel.is_cancelled() {
            summary.cancelled = true;

            report_download_progress(
                observer,
                &request.tax_code,
                request.invoice_type,
                TaskStatus::Cancelled,
                processed,
                Some(total),
                "Đã hủy quá trình tải hóa đơn",
                None,
                None,
                None,
            );

            return Ok(summary);
        }

        // Nếu có hóa đơn lưu lỗi thì không đánh dấu ngày hoàn thành.
        if summary.failed > 0 {
            report_download_progress(
                observer,
                &request.tax_code,
                request.invoice_type,
                TaskStatus::Failed,
                processed,
                Some(total),
                format!(
                    "Tải hóa đơn hoàn tất nhưng có {} hóa đơn lưu lỗi",
                    summary.failed
                ),
                None,
                Some(false),
                None,
            );

            return Ok(summary);
        }

        // Bước 4: Đánh dấu các ngày đã tải danh sách hoàn tất.
        for date in &dates {
            if self.cancel.is_cancelled() {
                summary.cancelled = true;
                break;
            }

            let is_today = date == &today_string;

            invoice_db.mark_day_list_completed(
                &request.tax_code,
                request.invoice_type,
                date,
                &is_today,
            )?;

            if is_today {
                continue;
            }

            invoice_db.delete_invoices_today_by_date(
                &request.tax_code,
                request.invoice_type,
                date,
            )?;
        }

        // Bước 5: Phát trạng thái kết thúc.
        if summary.cancelled || self.cancel.is_cancelled() {
            summary.cancelled = true;

            report_download_progress(
                observer,
                &request.tax_code,
                request.invoice_type,
                TaskStatus::Cancelled,
                processed,
                Some(total),
                "Đã hủy quá trình tải hóa đơn",
                None,
                None,
                None,
            );
        } else {
            report_download_progress(
                observer,
                &request.tax_code,
                request.invoice_type,
                TaskStatus::Completed,
                processed,
                Some(total),
                format!(
                    "Đã tải xong {} hóa đơn, lưu thành công {} hóa đơn",
                    summary.total, summary.saved
                ),
                None,
                Some(true),
                None,
            );
        }

        Ok(summary)
    }

    pub async fn retry_pending_details(
        &self,
        invoice_db: &InvoiceDatabase,
        tax_code: &str,
        invoice_type: u8,
        token: &str,
        batch_size: u32,
        observer: &dyn SyncObserver,
    ) -> Result<(usize, usize)> {
        if self.cancel.is_cancelled() {
            return Ok((0, 0));
        }

        // Bước 1: Lấy danh sách hóa đơn thiếu thông tin.
        report_detail_progress(
            observer,
            tax_code,
            TaskStatus::Running,
            0,
            0,
            "Đang tìm hóa đơn thiếu thông tin",
            invoice_type,
            None,
            None,
            None,
            None,
            None,
        );

        let invoices =
            invoice_db.get_invoices_pending_detail(tax_code, invoice_type, batch_size)?;

        let total = invoices.len();

        if total == 0 {
            report_detail_progress(
                observer,
                tax_code,
                TaskStatus::Completed,
                0,
                0,
                "Không có hóa đơn nào cần bổ sung thông tin",
                invoice_type,
                None,
                None,
                None,
                Some(true),
                None,
            );

            return Ok((0, 0));
        }

        // Bước 2: Bắt đầu bổ sung thông tin.
        report_detail_progress(
            observer,
            tax_code,
            TaskStatus::Running,
            0,
            total,
            format!("Tìm thấy {total} hóa đơn cần bổ sung thông tin"),
            invoice_type,
            None,
            None,
            None,
            None,
            None,
        );

        let mut completed = 0usize;
        let mut failed = 0usize;
        let mut processed = 0usize;

        for invoice in invoices {
            if self.cancel.is_cancelled() {
                break;
            }

            // Thông báo hóa đơn đang được xử lý.
            report_detail_progress(
                observer,
                tax_code,
                TaskStatus::Running,
                processed,
                total,
                format!("Đang bổ sung thông tin hóa đơn {}", invoice.shdon),
                invoice_type,
                None,
                Some(invoice.shdon),
                Some(invoice.khhdon.clone()),
                None,
                None,
            );

            let result = async {
                let (detail, xml) = self
                    .client
                    .fetch_detail_and_xml_by_key(
                        &invoice.nbmst,
                        &invoice.khhdon,
                        invoice.shdon,
                        invoice.khmshdon,
                        invoice_type,
                        token,
                    )
                    .await?;

                invoice_db.save_invoice_detail(
                    tax_code,
                    invoice_type,
                    &invoice.invoice_key,
                    &detail,
                    &xml,
                )?;

                Ok::<(), anyhow::Error>(())
            }
            .await;

            processed += 1;

            match result {
                Ok(()) => {
                    completed += 1;

                    self.upload_notify.notify_one();

                    report_detail_progress(
                        observer,
                        tax_code,
                        TaskStatus::Running,
                        processed,
                        total,
                        format!("Đã bổ sung thông tin hóa đơn {}", invoice.shdon),
                        invoice_type,
                        None,
                        Some(invoice.shdon),
                        Some(invoice.khhdon.clone()),
                        Some(true),
                        None,
                    );
                }

                Err(err) => {
                    if self.cancel.is_cancelled() {
                        break;
                    }

                    failed += 1;

                    let message = err.to_string();

                    if let Err(db_err) = invoice_db.set_detail_failed(
                        tax_code,
                        invoice_type,
                        &invoice.invoice_key,
                        &message,
                    ) {
                        eprintln!(
                        "[tax-sync] Cannot mark detail failed: tax_code={}, type={}, key={}, error={}",
                        tax_code, invoice_type, invoice.invoice_key, db_err
                    );
                    }

                    report_detail_progress(
                        observer,
                        tax_code,
                        TaskStatus::Running,
                        processed,
                        total,
                        format!("Bổ sung thông tin hóa đơn {} thất bại", invoice.shdon),
                        invoice_type,
                        None,
                        Some(invoice.shdon),
                        Some(invoice.khhdon.clone()),
                        Some(false),
                        Some(message),
                    );
                }
            }
        }

        // Bước 3: Kết thúc.
        if self.cancel.is_cancelled() {
            report_detail_progress(
                observer,
                tax_code,
                TaskStatus::Cancelled,
                processed,
                total,
                format!(
                    "Đã hủy bổ sung thông tin: thành công {}, thất bại {}",
                    completed, failed
                ),
                invoice_type,
                None,
                None,
                None,
                None,
                None,
            );
        } else if failed > 0 {
            report_detail_progress(
                observer,
                tax_code,
                TaskStatus::Failed,
                processed,
                total,
                format!(
                    "Bổ sung thông tin hoàn tất nhưng có {} hóa đơn thất bại",
                    failed
                ),
                invoice_type,
                None,
                None,
                None,
                Some(false),
                None,
            );
        } else {
            report_detail_progress(
                observer,
                tax_code,
                TaskStatus::Completed,
                processed,
                total,
                format!("Đã bổ sung thông tin thành công {} hóa đơn", completed),
                invoice_type,
                None,
                None,
                None,
                Some(true),
                None,
            );
        }

        Ok((completed, failed))
    }
}

fn dates_between(from: &str, to: &str) -> Result<Vec<String>> {
    let start = NaiveDate::parse_from_str(from, "%Y-%m-%d")?;
    let end = NaiveDate::parse_from_str(to, "%Y-%m-%d")?;

    if start > end {
        return Err(anyhow!("from_date must be <= to_date"));
    }

    let mut dates = Vec::new();
    let mut current = start;

    loop {
        dates.push(current.format("%Y-%m-%d").to_string());

        if current >= end {
            break;
        }

        current = current
            .succ_opt()
            .ok_or_else(|| anyhow!("Invalid date range"))?;
    }

    Ok(dates)
}

fn report_download_progress(
    observer: &dyn SyncObserver,
    tax_code: &str,
    invoice_type: u8,
    status: TaskStatus,
    completed: usize,
    total: Option<usize>,
    message: impl Into<String>,
    invoice: Option<&Value>,
    success: Option<bool>,
    error: Option<String>,
) {
    observer.on_progress(TaskProgress {
        tax_code: tax_code.to_string(),
        task: InvoiceTask::Download,
        status,
        completed,
        total,
        message: message.into(),
        invoice_type: invoice_type,
        invoice_date: None,
        invoice_number: None,
        invoice_serial: None,
        success,
        error,
        current_invoice: invoice.cloned(),
    });
}

fn report_detail_progress(
    observer: &dyn SyncObserver,
    tax_code: &str,
    status: TaskStatus,
    completed: usize,
    total: usize,
    message: impl Into<String>,
    invoice_type: u8,
    invoice_date: Option<String>,
    invoice_number: Option<i64>,
    invoice_serial: Option<String>,
    success: Option<bool>,
    error: Option<String>,
) {
    observer.on_progress(TaskProgress {
        tax_code: tax_code.to_string(),
        task: InvoiceTask::CompleteDetails,
        status,
        completed,
        total: Some(total),
        message: message.into(),
        invoice_type,
        invoice_date,
        invoice_number,
        invoice_serial,
        success,
        error,
        current_invoice: None,
    });
}
