use crate::auth::get_captcha;
use crate::db::invoice_database::{invoice_key, InvoiceDatabase};
use crate::error::TaxSyncError;
use crate::global_request_queue::GlobalRequestQueue;
use crate::models::FetchAllProgress;
use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{Datelike, Duration, NaiveDate};
use rand::RngExt;
use regex::Regex;
use reqwest::Client;
use serde_json::{json, Value};
use std::future::Future;
use std::io::{Cursor, Read};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::time::{sleep, Duration as TokioDuration};
use tokio_util::sync::CancellationToken;
use zip::ZipArchive;

const URL_EXPORT_XML: &str = "https://hoadondientu.gdt.gov.vn/api/query/invoices/export-xml";

const URL_SCO_EXPORT_XML: &str =
    "https://hoadondientu.gdt.gov.vn/api/sco-query/invoices/export-xml";

const PAGE_SIZE: usize = 50;

// pub async fn request_delay(delay_ms: u64) {
//     let duration =
//         rand::rng().random_range(delay_ms.saturating_sub(100)..=delay_ms.saturating_add(100));

//     tokio::time::sleep(std::time::Duration::from_millis(duration)).await;
// }

pub struct TaxClient {
    http: Client,
    tax_code: String,
    request_id: String,
    request_queue: Arc<GlobalRequestQueue>,
    cancel: CancellationToken,
}

impl TaxClient {
    pub fn new(
        tax_code: String,
        request_queue: Arc<GlobalRequestQueue>,
        cancel: CancellationToken,
    ) -> Result<Self> {
        let http = Client::builder()
            .cookie_store(true)
            .timeout(TokioDuration::from_secs(60))
            .build()?;
        let request_id = uuid::Uuid::new_v4().to_string();
        Ok(Self {
            http,
            tax_code,
            request_id,
            request_queue,
            cancel,
        })
    }

    fn invoice_url(invoice_type: u8, from: &str, to: &str) -> Result<String> {
        let search = format!("tdlap=ge={}T00:00:00;tdlap=le={}T23:59:59", from, to);

        let base = match invoice_type {
            1 => "https://hoadondientu.gdt.gov.vn/api/query/invoices/purchase",
            2 => "https://hoadondientu.gdt.gov.vn/api/query/invoices/sold",
            3 => "https://hoadondientu.gdt.gov.vn/api/sco-query/invoices/purchase",
            4 => "https://hoadondientu.gdt.gov.vn/api/sco-query/invoices/sold",
            _ => bail!("Invalid invoice type: {}", invoice_type),
        };

        let search = if invoice_type == 1 || invoice_type == 3 {
            format!("{};ttxly==5", search)
        } else {
            search
        };

        Ok(format!(
            "{}?sort=shdon:desc&size={}&search={}",
            base,
            PAGE_SIZE,
            url::form_urlencoded::byte_serialize(search.as_bytes()).collect::<String>()
        ))
    }

    async fn get_bytes(
        &self,
        url: &str,
        query: &[(&str, String)],
        token: &str,
    ) -> anyhow::Result<Vec<u8>> {
        let url = url.to_string();
        let query = query.to_vec();
        let token = token.to_string();

        let http = self.http.clone();
        let queue = self.request_queue.clone();
        let queue_for_429 = queue.clone();
        let tax_code = self.tax_code.clone();
        queue
            .execute(&tax_code.clone(), &self.cancel, || async move {
                let response = http
                    .get(&url)
                    .header("request-id", uuid::Uuid::new_v4().to_string())
                    .query(&query)
                    .bearer_auth(&token)
                    .send()
                    .await?;

                let status = response.status();

                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    let retry_after_secs = response
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(180)
                        .max(180);

                    let wait = TokioDuration::from_secs(retry_after_secs);

                    queue_for_429
                        .set_tax_code_cooldown(&tax_code.clone(), wait)
                        .await;

                    return Err(TaxSyncError::RateLimited {
                        tax_code,
                        retry_after_secs,
                    }
                    .into());
                }

                if !status.is_success() {
                    anyhow::bail!("Tax API returned HTTP {}", status);
                }

                let bytes = response.bytes().await?.to_vec();
                Ok(bytes)
            })
            .await
    }

    async fn get_json(
        &self,
        url: &str,
        token: &str,
        headers: Option<&[(&str, &str)]>,
    ) -> anyhow::Result<Value> {
        let url = url.to_string();
        let token = token.to_string();
        let headers = headers.map(|h| h.to_vec());

        let http = self.http.clone();
        let queue = self.request_queue.clone();
        let queue_for_429 = queue.clone();
        let tax_code = self.tax_code.clone();

        queue
            .execute(&tax_code.clone(), &self.cancel, || async move {
                let mut request = http
                    .get(&url)
                    .header("request-id", uuid::Uuid::new_v4().to_string())
                    .bearer_auth(&token);

                if let Some(headers) = headers {
                    for (name, value) in headers {
                        request = request.header(name, value);
                    }
                }

                let response = request.send().await?;
                let status = response.status();

                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    let retry_after_secs = response
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(180)
                        .max(180);

                    let wait = TokioDuration::from_secs(retry_after_secs);

                    queue_for_429
                        .set_tax_code_cooldown(&tax_code.clone(), wait)
                        .await;

                    return Err(TaxSyncError::RateLimited {
                        tax_code,
                        retry_after_secs,
                    }
                    .into());
                }

                if !status.is_success() {
                    anyhow::bail!("Tax API returned HTTP {}", status);
                }

                response
                    .json::<Value>()
                    .await
                    .context("Invalid tax API JSON response")
            })
            .await
    }

    pub async fn fetch_all<F, Fut, P>(
        &self,
        tax_code: &str,
        invoice_type: u8,
        from_date: &str,
        to_date: &str,
        token: &str,
        invoice_db: &InvoiceDatabase,
        solve_captcha: F,
        mut on_progress: P,
    ) -> Result<Vec<Value>>
    where
        F: Fn(&str) -> Fut,
        Fut: Future<Output = Result<String>>,
        P: FnMut(FetchAllProgress),
    {
        let mut result = Vec::new();

        let mut current = NaiveDate::parse_from_str(from_date, "%Y-%m-%d")?;
        let end = NaiveDate::parse_from_str(to_date, "%Y-%m-%d")?;

        if current > end {
            return Err(
                TaxSyncError::InvalidDateRange("from_date must be <= to_date".to_string()).into(),
            );
        }

        // Tính tổng số tháng cần đồng bộ.
        let mut total_months = 0;
        let mut temp_date = current;

        while temp_date <= end {
            total_months += 1;

            temp_date = if temp_date.month() == 12 {
                NaiveDate::from_ymd_opt(temp_date.year() + 1, 1, 1).unwrap()
            } else {
                NaiveDate::from_ymd_opt(temp_date.year(), temp_date.month() + 1, 1).unwrap()
            };
        }

        let mut completed_months = 0;

        while current <= end {
            if self.cancel.is_cancelled() {
                return Err(TaxSyncError::Api("Sync cancelled".to_string()).into());
            }

            let first_next_month = if current.month() == 12 {
                NaiveDate::from_ymd_opt(current.year() + 1, 1, 1).unwrap()
            } else {
                NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1).unwrap()
            };

            let month_end = (first_next_month - Duration::days(1)).min(end);

            let from = current.format("%d/%m/%Y").to_string();
            let to = month_end.format("%d/%m/%Y").to_string();

            let base_url = Self::invoice_url(invoice_type, &from, &to)?;

            let mut state: Option<String> = None;
            let mut page = 0;

            // Thông báo bắt đầu tải tháng.
            on_progress(FetchAllProgress {
                completed_months,
                total_months,
                total_invoices: result.len(),
                current_from: from.clone(),
                current_to: to.clone(),
                page,
                message: format!("Bắt đầu tải hóa đơn ngày {}", current.format("%d/%m/%Y"),),
            });

            loop {
                if self.cancel.is_cancelled() {
                    return Err(TaxSyncError::Api("Sync cancelled".to_string()).into());
                }

                page += 1;

                // Thông báo đang tải trang API.
                on_progress(FetchAllProgress {
                    completed_months,
                    total_months,
                    total_invoices: result.len(),
                    current_from: from.clone(),
                    current_to: to.clone(),
                    page,
                    message: format!(
                        "Đang tải trang {} ngày {}",
                        page,
                        current.format("%d/%m/%Y"),
                    ),
                });

                let captcha = if invoice_type == 3 || invoice_type == 4 {
                    let captcha = get_captcha(&self.http).await?;

                    // Giải captcha bằng hàm do ứng dụng cung cấp.
                    let captcha_value = solve_captcha(&captcha.cvalue).await?;
                    let captcha_value = captcha_value.trim();

                    if captcha_value.is_empty() {
                        return Err(TaxSyncError::Authentication(
                            "Đáp án captcha rỗng".to_string(),
                        )
                        .into());
                    }

                    Some((captcha.ckey, captcha_value.to_string()))
                } else {
                    None
                };

                let captcha_headers: Vec<(&str, &str)> = match captcha.as_ref() {
                    Some((ckey, cvalue)) => {
                        vec![("ckey", ckey.as_str()), ("cvalue", cvalue.as_str())]
                    }
                    None => vec![],
                };

                let mut url = url::Url::parse(&base_url)
                    .map_err(|e| TaxSyncError::Authentication(e.to_string()))?;

                {
                    let mut query = url.query_pairs_mut();

                    if let Some(cursor) = state.as_ref() {
                        query.append_pair("state", cursor);
                    }

                    if let Some((ckey, cvalue)) = captcha.as_ref() {
                        query.append_pair("ckey", ckey);
                        query.append_pair("cvalue", cvalue);
                    }
                }

                let request_url = url.to_string();

                // Gọi API.
                let response = match self
                    .get_json(&request_url, token, Some(&captcha_headers))
                    .await
                {
                    Ok(response) => response,

                    Err(err) => {
                        eprintln!("Request URL: {request_url}");
                        eprintln!("Captcha headers: {:?}", captcha_headers);

                        return Err(err);
                    }
                };

                if self.cancel.is_cancelled() {
                    return Err(TaxSyncError::Api("Sync cancelled".to_string()).into());
                }

                // Nhận dữ liệu hóa đơn của trang hiện tại.
                let page_count = response["datas"]
                    .as_array()
                    .map(|items| items.len())
                    .unwrap_or(0);

                if let Some(items) = response["datas"].as_array() {
                    // eprintln!("invoices: {:?}", items);
                    if let Some(item) = items.last() {
                        let invoice_key = invoice_key(item)?;

                        let exists = invoice_db.invoice_today_exists(
                            tax_code,
                            invoice_type,
                            &invoice_key,
                        )?;

                        if exists {
                            break;
                        }
                    }
                    result.extend(items.iter().cloned());
                }

                // Cập nhật tiến trình sau mỗi trang.
                on_progress(FetchAllProgress {
                    completed_months,
                    total_months,
                    total_invoices: result.len(),
                    current_from: from.clone(),
                    current_to: to.clone(),
                    page,
                    message: format!(
                        "Đã tải trang {} ngày {}. Nhận thêm {} hóa đơn",
                        page,
                        current.format("%d/%m/%Y"),
                        page_count,
                    ),
                });

                // Kiểm tra cursor phân trang.
                match response["state"].as_str() {
                    Some(next) if !next.is_empty() => {
                        state = Some(next.to_owned());
                    }
                    _ => break,
                }
            }

            // Hoàn thành tháng hiện tại.
            completed_months += 1;

            on_progress(FetchAllProgress {
                completed_months,
                total_months,
                total_invoices: result.len(),
                current_from: from.clone(),
                current_to: to.clone(),
                page,
                message: format!(
                    "Đã tải xong ngày {}. Tổng cộng {} hóa đơn",
                    current.format("%d/%m/%Y"),
                    result.len(),
                ),
            });

            current = first_next_month;
        }

        if self.cancel.is_cancelled() {
            return Err(TaxSyncError::Api("Sync cancelled".to_string()).into());
        }

        // Sắp xếp hóa đơn theo ngày lập.
        result.sort_by(|a, b| {
            a["tdlap"]
                .as_str()
                .unwrap_or("")
                .cmp(b["tdlap"].as_str().unwrap_or(""))
        });

        Ok(result)
    }

    pub async fn fetch_detail_and_xml(
        &self,
        invoice: &Value,
        token: &str,
    ) -> Result<(Option<Value>, Option<String>)> {
        let xml = self.fetch_xml(invoice, token).await?;

        if xml.is_some() {
            return Ok((None, xml));
        }

        let detail = self.fetch_detail(invoice, token).await?;

        Ok((Some(detail), None))
    }

    async fn fetch_detail(&self, invoice: &Value, token: &str) -> Result<Value> {
        let nbmst: &str = invoice["nbmst"].as_str().context("Missing nbmst")?;

        let khhdon = invoice["khhdon"].as_str().context("Missing khhdon")?;

        let shdon = invoice["shdon"].as_i64().context("Missing shdon")?;

        let khmshdon = invoice["khmshdon"].as_i64().context("Missing khmshdon")?;

        self.fetch_detail_by_key(nbmst, khhdon, shdon, khmshdon, token)
            .await
    }

    async fn fetch_xml(&self, invoice: &Value, token: &str) -> Result<Option<String>> {
        let nbmst = invoice["nbmst"].as_str().context("Missing nbmst")?;

        let khhdon = invoice["khhdon"].as_str().context("Missing khhdon")?;

        let shdon = invoice["shdon"].as_i64().context("Missing shdon")?;

        let khmshdon = invoice["khmshdon"].as_i64().context("Missing khmshdon")?;

        let invoice_type = u8::try_from(
            invoice["invoice_type"]
                .as_u64()
                .context("Missing invoice_type")?,
        )
        .context("invoice_type vượt phạm vi u8")?;

        self.fetch_xml_by_key(nbmst, khhdon, shdon, khmshdon, invoice_type, token)
            .await
    }

    pub async fn fetch_detail_and_xml_by_key(
        &self,
        nbmst: &str,
        khhdon: &str,
        shdon: i64,
        khmshdon: i64,
        invoice_type: u8,
        token: &str,
    ) -> Result<(Option<Value>, Option<String>)> {
        let xml = self
            .fetch_xml_by_key(nbmst, khhdon, shdon, khmshdon, invoice_type, token)
            .await?;

        if xml.is_some() {
            return Ok((None, xml));
        }

        let detail = self
            .fetch_detail_by_key(nbmst, khhdon, shdon, khmshdon, token)
            .await?;

        Ok((Some(detail), None))
    }

    async fn fetch_detail_by_key(
        &self,
        nbmst: &str,
        khhdon: &str,
        shdon: i64,
        khmshdon: i64,
        token: &str,
    ) -> Result<Value> {
        let mut url =
            reqwest::Url::parse("https://hoadondientu.gdt.gov.vn/api/query/invoices/detail")?;

        url.query_pairs_mut()
            .append_pair("nbmst", nbmst)
            .append_pair("khhdon", khhdon)
            .append_pair("shdon", &shdon.to_string())
            .append_pair("khmshdon", &khmshdon.to_string());

        self.get_json(url.as_str(), token, None).await
    }

    pub async fn fetch_xml_by_key(
        &self,
        nbmst: &str,
        khhdon: &str,
        shdon: i64,
        khmshdon: i64,
        invoice_type: u8,
        token: &str,
    ) -> Result<Option<String>> {
        let base_url = match invoice_type {
            1 | 2 => URL_EXPORT_XML,
            3 | 4 => URL_SCO_EXPORT_XML,
            _ => {
                eprintln!("[tax-sync] Loại hóa đơn không hợp lệ: {invoice_type}");
                return Ok(None);
            }
        };

        let result: Result<Option<String>> = async {
            let bytes = self
                .get_bytes(
                    base_url,
                    &[
                        ("nbmst", nbmst.to_string()),
                        ("khhdon", khhdon.to_string()),
                        ("shdon", shdon.to_string()),
                        ("khmshdon", khmshdon.to_string()),
                    ],
                    token,
                )
                .await?;

            if bytes.len() < 4 || bytes[0..2] != [0x50, 0x4B] {
                return Err(TaxSyncError::Api(format!(
                    "Response không phải ZIP hợp lệ, shdon={shdon}"
                ))
                .into());
            }

            let cursor = Cursor::new(bytes);

            let mut archive = ZipArchive::new(cursor).map_err(|err| {
                TaxSyncError::Api(format!("Không đọc được ZIP, shdon={shdon}: {err}"))
            })?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|err| {
                    TaxSyncError::Api(format!(
                        "Không đọc được file trong ZIP, shdon={shdon}: {err}"
                    ))
                })?;

                if file.name().to_ascii_lowercase().ends_with(".xml") {
                    let mut content = String::new();

                    file.read_to_string(&mut content).map_err(|err| {
                        TaxSyncError::Api(format!("Không đọc được XML, shdon={shdon}: {err}"))
                    })?;
                    return Ok(Some(content));
                    // return Ok(Some(STANDARD.encode(content.as_bytes())));
                }
            }

            Err(TaxSyncError::Api(format!("Không tìm thấy XML trong ZIP, shdon={shdon}")).into())
        }
        .await;

        if let Err(ref err) = result {
            eprintln!("[tax-sync] Lỗi khi lấy XML, shdon={shdon}: {err}");
        }

        result
    }
}
