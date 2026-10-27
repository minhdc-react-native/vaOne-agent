use crate::client::request_delay;
use crate::database::{invoice_date_vietnam, invoice_key};
use crate::error::TaxSyncError;
use crate::models::TaxAccount;
use crate::worker::{vietnam_today, CaptchaSolver};
use crate::{
    client::TaxClient,
    database::Database,
    models::{InvoiceProgress, SyncRequest, SyncSummary},
};

use anyhow::{anyhow, Result};

use chrono::NaiveDate;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;
pub trait SyncObserver: Send + Sync {
    fn on_started(&self, total: usize);
    fn on_invoice(&self, progress: InvoiceProgress);
    fn on_finished(&self, summary: &SyncSummary);
    fn on_notification(&self, tax_accounts: &Vec<TaxAccount>);
}

pub struct TaxSyncService {
    client: TaxClient,
    db: Database,
    upload_notify: Arc<Notify>,
    update_notify: Arc<Notify>,
    captcha_solver: Arc<dyn CaptchaSolver>,
}

impl TaxSyncService {
    pub fn new(
        db: Database,
        upload_notify: Arc<Notify>,
        update_notify: Arc<Notify>,
        captcha_solver: Arc<dyn CaptchaSolver>,
    ) -> Result<Self> {
        Ok(Self {
            client: TaxClient::new()?,
            db,
            upload_notify,
            update_notify,
            captcha_solver,
        })
    }

    pub async fn run(
        &self,
        request: SyncRequest,
        observer: &dyn SyncObserver,
        cancel: CancellationToken,
    ) -> Result<SyncSummary> {
        if request.tax_code.trim().is_empty() {
            return Err(
                TaxSyncError::InvalidConfig("tax_code must not be empty".to_string()).into(),
            );
        }

        if request.token.trim().is_empty() {
            return Err(TaxSyncError::InvalidConfig("token must not be empty".to_string()).into());
        }

        if cancel.is_cancelled() {
            return Ok(SyncSummary {
                cancelled: true,
                ..Default::default()
            });
        }

        let dates = dates_between(&request.from_date, &request.to_date)?;

        let solver = Arc::clone(&self.captcha_solver);

        // Bước 1: Lấy danh sách hóa đơn.
        let invoices = match self
            .client
            .fetch_all(
                request.invoice_type,
                &request.from_date,
                &request.to_date,
                &request.token,
                request.delay_ms,
                cancel.clone(),
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
            )
            .await
        {
            Ok(invoices) => invoices,

            Err(err) => {
                if cancel.is_cancelled() {
                    return Ok(SyncSummary {
                        cancelled: true,
                        ..Default::default()
                    });
                }

                let message = err.to_string();

                for date in &dates {
                    let _ = self.db.set_day_list_failed(
                        &request.tax_code,
                        request.invoice_type,
                        date,
                        &message,
                    );
                }

                return Err(err);
            }
        };

        if cancel.is_cancelled() {
            return Ok(SyncSummary {
                cancelled: true,
                ..Default::default()
            });
        }

        observer.on_started(invoices.len());

        // Bước 2: Lưu summary.
        let mut summary = SyncSummary {
            total: invoices.len(),
            ..Default::default()
        };

        let today = vietnam_today().await?;

        for item in &invoices {
            if cancel.is_cancelled() {
                summary.cancelled = true;
                break;
            }

            let invoice_key = invoice_key(item)?;

            let exists = self.db.invoice_today_exists(
                &request.tax_code,
                request.invoice_type,
                &invoice_key,
            )?;

            if exists {
                continue;
            }

            if cancel.is_cancelled() {
                summary.cancelled = true;
                break;
            }

            let result = (|| -> Result<(String, String)> {
                let date = invoice_date_vietnam(item)?;

                let is_today = date == today.format("%Y-%m-%d").to_string();

                let key = self.db.save_invoice_summary(
                    &request.tax_code,
                    request.invoice_type,
                    item,
                    is_today,
                )?;

                self.update_notify.notify_one();

                Ok((date, key))
            })();

            // Giữ nguyên logic xử lý result của bạn tại đây:
            // cập nhật summary.saved / summary.failed,
            // ghi log lỗi nếu lưu summary thất bại.
            //
            // Nếu result.is_err(), nhớ tăng summary.failed.
            let _ = result;
        }

        // Không hoàn tất ngày nếu đã hủy hoặc có lỗi lưu summary.
        if summary.cancelled || summary.failed > 0 {
            observer.on_finished(&summary);
            return Ok(summary);
        }

        // Kiểm tra lần cuối trước khi đánh dấu completed.
        if cancel.is_cancelled() {
            summary.cancelled = true;
            observer.on_finished(&summary);
            return Ok(summary);
        }

        // Bước 3: Chỉ hoàn thành khi đã lưu đủ danh sách.
        for date in &dates {
            if cancel.is_cancelled() {
                summary.cancelled = true;
                break;
            }

            let is_today = date == today.format("%Y-%m-%d").to_string().as_str();

            self.db.mark_day_list_completed(
                &request.tax_code,
                request.invoice_type,
                date,
                &is_today,
            )?;
            if (is_today) {
                continue;
            }
            self.db
                .delete_invoices_today_by_date(&request.tax_code, request.invoice_type, date)?;
        }

        observer.on_finished(&summary);

        Ok(summary)
    }

    pub async fn retry_pending_details(
        &self,
        tax_code: &str,
        invoice_type: u8,
        token: &str,
        delay_ms: u64,
        batch_size: u32,
        cancel: CancellationToken,
    ) -> Result<(usize, usize)> {
        if cancel.is_cancelled() {
            return Ok((0, 0));
        }

        let invoices = self
            .db
            .get_invoices_pending_detail(tax_code, invoice_type, batch_size)?;

        let mut completed = 0;
        let mut failed = 0;
        for invoice in invoices {
            if cancel.is_cancelled() {
                break;
            }

            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = request_delay(delay_ms) => {}
            }

            if cancel.is_cancelled() {
                break;
            }

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
                        delay_ms,
                    )
                    .await?;

                self.db.save_invoice_detail(
                    tax_code,
                    invoice_type,
                    &invoice.invoice_key,
                    &detail,
                    &xml,
                )?;

                Ok::<(), anyhow::Error>(())
            }
            .await;

            match result {
                Ok(()) => {
                    completed += 1;
                    self.upload_notify.notify_one();
                    // println!(
                    //     "[tax-sync] Detail retry success: tax_code={}, invoice={}",
                    //     tax_code, invoice.shdon
                    // );
                }
                Err(err) => {
                    if cancel.is_cancelled() {
                        break;
                    }

                    failed += 1;

                    let message = err.to_string();

                    // Chỉ cập nhật trạng thái lỗi, không trả lỗi ra ngoài.
                    if let Err(db_err) = self.db.set_detail_failed(
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
                }
            }
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
