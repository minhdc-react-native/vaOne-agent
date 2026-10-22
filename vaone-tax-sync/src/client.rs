use crate::error::TaxSyncError;
use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::{Datelike, Duration, NaiveDate};
use rand::RngExt;
use regex::Regex;
use reqwest::Client;
use serde_json::{json, Value};
use std::io::{Cursor, Read};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::time::{sleep, Duration as TokioDuration};
use zip::ZipArchive;

const URL_EXPORT_XML: &str = "https://hoadondientu.gdt.gov.vn/api/query/invoices/export-xml";

const URL_SCO_EXPORT_XML: &str =
    "https://hoadondientu.gdt.gov.vn/api/sco-query/invoices/export-xml";

const PAGE_SIZE: usize = 50;

pub async fn request_delay(delay_ms: u64) {
    let duration =
        rand::rng().random_range(delay_ms.saturating_sub(100)..=delay_ms.saturating_add(100));

    tokio::time::sleep(std::time::Duration::from_millis(duration)).await;
}

pub struct TaxClient {
    http: Client,
    request_id: String,
}

impl TaxClient {
    pub fn new() -> Result<Self> {
        let http = Client::builder()
            .timeout(TokioDuration::from_secs(60))
            .build()?;
        let request_id = uuid::Uuid::new_v4().to_string();
        Ok(Self { http, request_id })
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
            "{}?sort=tdlap:desc&size={}&search={}",
            base,
            PAGE_SIZE,
            url::form_urlencoded::byte_serialize(search.as_bytes()).collect::<String>()
        ))
    }

    async fn get_json(&self, url: &str, token: &str) -> Result<Value> {
        let response = self
            .http
            .get(url)
            .header("request-id", self.request_id.clone())
            .bearer_auth(token)
            .send()
            .await
            .context("Tax API request failed")?;

        let status = response.status();

        if !status.is_success() {
            bail!("Tax API returned HTTP {}", status);
        }

        response
            .json::<Value>()
            .await
            .context("Invalid tax API JSON response")
    }

    pub async fn fetch_all(
        &self,
        invoice_type: u8,
        from_date: &str,
        to_date: &str,
        token: &str,
        delay_ms: u64,
        cancel: Arc<AtomicBool>,
    ) -> Result<Vec<Value>> {
        let mut result = Vec::new();

        let mut current = NaiveDate::parse_from_str(from_date, "%Y-%m-%d")?;
        let end = NaiveDate::parse_from_str(to_date, "%Y-%m-%d")?;

        if current > end {
            return Err(
                TaxSyncError::InvalidDateRange("from_date must be <= to_date".to_string()).into(),
            );
        }
        while current <= end {
            if cancel.load(Ordering::Relaxed) {
                break;
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

            loop {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                request_delay(delay_ms).await;
                let mut request_url = base_url.clone();

                if let Some(cursor) = &state {
                    let separator = if request_url.contains('?') { '&' } else { '?' };

                    request_url.push(separator);
                    request_url.push_str("state=");
                    request_url.push_str(
                        &url::form_urlencoded::byte_serialize(cursor.as_bytes())
                            .collect::<String>(),
                    );
                }

                let response = self.get_json(&request_url, token).await?;

                if let Some(items) = response["datas"].as_array() {
                    result.extend(items.iter().cloned());
                }

                match response["state"].as_str() {
                    Some(next) if !next.is_empty() => {
                        state = Some(next.to_owned());
                    }
                    _ => break,
                }
            }

            current = first_next_month;
        }

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

        request_delay(2000).await;

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
        request_delay(2000).await;
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

        self.get_json(url.as_str(), token).await
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
            let response = self
                .http
                .get(base_url)
                .header("request-id", self.request_id.clone())
                .query(&[
                    ("nbmst", nbmst),
                    ("khhdon", khhdon),
                    ("shdon", &shdon.to_string()),
                    ("khmshdon", &khmshdon.to_string()),
                ])
                .bearer_auth(token)
                .send()
                .await?;

            let status = response.status();

            if !status.is_success() {
                return Err(TaxSyncError::Api(format!(
                    "Lấy XML thất bại: HTTP {status}, shdon={shdon}"
                ))
                .into());
            }

            let bytes = response.bytes().await?;

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
