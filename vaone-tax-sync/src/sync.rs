use crate::client::request_delay;
use crate::database::invoice_key;
use crate::error::TaxSyncError;
use crate::worker::vietnam_today;
use crate::{
    client::TaxClient,
    database::Database,
    models::{InvoiceProgress, SyncRequest, SyncSummary},
};

use anyhow::{anyhow, Result};

use chrono::{DateTime, FixedOffset, NaiveDate};
use serde_json::Value;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio_util::sync::CancellationToken;
pub trait SyncObserver: Send + Sync {
    fn on_started(&self, total: usize);
    fn on_invoice(&self, progress: InvoiceProgress);
    fn on_finished(&self, summary: &SyncSummary);
}

pub struct TaxSyncService {
    client: TaxClient,
    db: Database,
}

impl TaxSyncService {
    pub fn new(db: Database) -> Result<Self> {
        Ok(Self {
            client: TaxClient::new()?,
            db,
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

        for date in &dates {
            if cancel.is_cancelled() {
                return Ok(SyncSummary {
                    cancelled: true,
                    ..Default::default()
                });
            }

            self.db
                .ensure_sync_day(&request.tax_code, request.invoice_type, date)?;

            self.db
                .set_day_running(&request.tax_code, request.invoice_type, date)?;
        }

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
            )
            .await
        {
            Ok(invoices) => invoices,

            Err(err) => {
                // Phân biệt cancellation với lỗi đồng bộ thực sự.
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

            let today = vietnam_today().await?;

            if cancel.is_cancelled() {
                summary.cancelled = true;
                break;
            }

            let result = (|| -> Result<(String, String)> {
                let date = invoice_date_vietnam(item)?;

                self.db
                    .ensure_sync_day(&request.tax_code, request.invoice_type, &date)?;

                let is_today = date == today.format("%Y-%m-%d").to_string();

                let key = self.db.save_invoice_summary(
                    &request.tax_code,
                    request.invoice_type,
                    item,
                    is_today,
                )?;

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

            self.db
                .mark_day_list_completed(&request.tax_code, request.invoice_type, date)?;

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

            // Có thể dừng ngay trong thời gian chờ, không phải chờ hết delay.
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = request_delay(delay_ms) => {}
            }

            // Kiểm tra lại sau delay, trước khi bắt đầu request.
            if cancel.is_cancelled() {
                break;
            }

            // Không hủy request đang chạy giữa chừng.
            // Chờ kết quả để cập nhật SQLite nhất quán.
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
                }
                Err(err) => {
                    // Nếu worker bị hủy, không tự đánh dấu hóa đơn thất bại
                    // chỉ vì tiến trình cần dừng.
                    if cancel.is_cancelled() {
                        break;
                    }

                    failed += 1;

                    let message = err.to_string();

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

                    eprintln!(
                        "[tax-sync] Detail retry failed: tax_code={}, type={}, key={}: {}",
                        tax_code, invoice_type, invoice.invoice_key, message
                    );
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

fn invoice_date_vietnam(invoice: &Value) -> Result<String> {
    let timestamp = invoice["tdlap"]
        .as_str()
        .ok_or_else(|| anyhow!("Invoice is missing tdlap"))?;

    let parsed = DateTime::parse_from_rfc3339(timestamp)?;

    let vietnam_offset =
        FixedOffset::east_opt(7 * 60 * 60).ok_or_else(|| anyhow!("Invalid Vietnam timezone"))?;

    Ok(parsed
        .with_timezone(&vietnam_offset)
        .format("%Y-%m-%d")
        .to_string())
}
