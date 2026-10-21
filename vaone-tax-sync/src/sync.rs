use crate::client::request_delay;
use crate::error::TaxSyncError;
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
        cancel: Arc<AtomicBool>,
    ) -> Result<SyncSummary> {
        if request.tax_code.trim().is_empty() {
            return Err(
                TaxSyncError::InvalidConfig("tax_code must not be empty".to_string()).into(),
            );
        }

        if request.token.trim().is_empty() {
            return Err(TaxSyncError::InvalidConfig("token must not be empty".to_string()).into());
        }

        let dates = dates_between(&request.from_date, &request.to_date)?;

        for date in &dates {
            self.db
                .ensure_sync_day(&request.tax_code, request.invoice_type, date)?;

            self.db
                .set_day_running(&request.tax_code, request.invoice_type, date)?;
        }

        if cancel.load(Ordering::Relaxed) {
            return Ok(SyncSummary {
                cancelled: true,
                ..Default::default()
            });
        }

        // Bước 1: Lấy toàn bộ danh sách, bao gồm các trang tiếp theo.
        let invoices = match self
            .client
            .fetch_all(
                request.invoice_type,
                &request.from_date,
                &request.to_date,
                &request.token,
                request.delay_ms,
                Arc::clone(&cancel),
            )
            .await
        {
            Ok(invoices) => invoices,
            Err(err) => {
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

        if cancel.load(Ordering::Relaxed) {
            return Ok(SyncSummary {
                total: invoices.len(),
                cancelled: true,
                ..Default::default()
            });
        }

        observer.on_started(invoices.len());

        // Bước 2: Lưu các bản ghi tóm tắt.
        let mut summary = SyncSummary {
            total: invoices.len(),
            ..Default::default()
        };

        for item in &invoices {
            if cancel.load(Ordering::Relaxed) {
                summary.cancelled = true;
                break;
            }

            // Bước 1: Lưu summary trước.
            let result = (|| -> Result<(String, String)> {
                let date = invoice_date_vietnam(item)?;

                self.db
                    .ensure_sync_day(&request.tax_code, request.invoice_type, &date)?;

                let key =
                    self.db
                        .save_invoice_summary(&request.tax_code, request.invoice_type, item)?;

                Ok((date, key))
            })();

            match result {
                Ok((_date, invoice_key)) => {
                    summary.completed += 1;

                    // Bước 2: Tải chi tiết hóa đơn.
                    let (detail, xml) = match self
                        .client
                        .fetch_detail_and_xml(&item, &request.token)
                        .await
                    {
                        Ok(result) => result,
                        Err(err) => {
                            let message = err.to_string();

                            let _ = self.db.set_detail_failed(
                                &request.tax_code,
                                request.invoice_type,
                                &invoice_key,
                                &message,
                            );

                            eprintln!(
                                "[tax-sync] Fetch detail/XML failed: tax_code={}, type={}, key={}: {}",
                                request.tax_code,
                                request.invoice_type,
                                invoice_key,
                                message
                            );

                            continue;
                        }
                    };

                    if let Err(err) = self.db.save_invoice_detail(
                        &request.tax_code,
                        request.invoice_type,
                        &invoice_key,
                        &detail,
                        &xml,
                    ) {
                        let message = err.to_string();

                        let _ = self.db.set_detail_failed(
                            &request.tax_code,
                            request.invoice_type,
                            &invoice_key,
                            &message,
                        );

                        eprintln!(
                            "[tax-sync] Save detail failed: tax_code={}, type={}, key={}: {}",
                            request.tax_code, request.invoice_type, invoice_key, message
                        );

                        continue;
                    }

                    // Bước 4: Báo tiến trình.
                    observer.on_invoice(InvoiceProgress {
                        completed: summary.completed,
                        total: summary.total,
                        invoice_date: item["tdlap"].as_str().unwrap_or("").to_owned(),
                        invoice_number: item["shdon"].as_i64().unwrap_or(0),
                        invoice_serial: item["khhdon"].as_str().unwrap_or("").to_owned(),
                        success: true,
                        error: None,
                        current_invoice: Some(item.clone()),
                    });
                }

                Err(err) => {
                    summary.failed += 1;

                    observer.on_invoice(InvoiceProgress {
                        completed: summary.completed + summary.failed,
                        total: summary.total,
                        invoice_date: item["tdlap"].as_str().unwrap_or("").to_owned(),
                        invoice_number: item["shdon"].as_i64().unwrap_or(0),
                        invoice_serial: item["khhdon"].as_str().unwrap_or("").to_owned(),
                        success: false,
                        error: Some(err.to_string()),
                        current_invoice: Some(item.clone()),
                    });

                    let message = err.to_string();

                    for date in &dates {
                        let _ = self.db.set_day_list_failed(
                            &request.tax_code,
                            request.invoice_type,
                            date,
                            &message,
                        );
                    }

                    observer.on_finished(&summary);
                    return Err(err);
                }
            }
        }

        // Bước 3: Chỉ hoàn thành nếu đã lưu hết danh sách.
        if summary.cancelled || summary.failed > 0 {
            observer.on_finished(&summary);

            return Ok(summary);
        }

        for date in &dates {
            self.db
                .mark_day_list_completed(&request.tax_code, request.invoice_type, date)?;
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
        cancel: Arc<AtomicBool>,
    ) -> Result<(usize, usize)> {
        let invoices = self
            .db
            .get_invoices_pending_detail(tax_code, invoice_type, batch_size)?;

        let mut completed = 0;
        let mut failed = 0;

        for invoice in invoices {
            if cancel.load(Ordering::Relaxed) {
                break;
            }

            request_delay(delay_ms).await;

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
                Ok(()) => completed += 1,
                Err(err) => {
                    failed += 1;

                    let message = err.to_string();

                    let _ = self.db.set_detail_failed(
                        tax_code,
                        invoice_type,
                        &invoice.invoice_key,
                        &message,
                    );

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
