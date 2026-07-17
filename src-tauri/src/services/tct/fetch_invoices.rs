use chrono::{Datelike, Duration, NaiveDate};
use serde_json::Value;
use std::cmp::Ordering as CmpOrdering;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::utils::public::navigate_to_route;

fn get_url(invoice_type: u8, from_date: &str, to_date: &str) -> String {
    let size_page = 50;

    let search = format!(
        "tdlap=ge={}T00:00:00;tdlap=le={}T23:59:59",
        from_date, to_date
    );

    match invoice_type {
        // 1: Mua vào
        // 3: Mua vào MTT
        1 => format!(
            "https://hoadondientu.gdt.gov.vn/api/query/invoices/purchase?sort=tdlap:desc&size={}&search={};ttxly==5",
            size_page,
            search
        ),
        3 => format!(
            "https://hoadondientu.gdt.gov.vn/api/sco-query/invoices/purchase?sort=tdlap:desc&size={}&search={};ttxly==5",
            size_page,
            search
        ),
        // 2: Bán ra
        // 4: Bán ra MTT
        2 => format!(
            "https://hoadondientu.gdt.gov.vn/api/query/invoices/sold?sort=tdlap:desc&size={}&search={}",
            size_page,
            search
        ),
        4 => format!(
            "https://hoadondientu.gdt.gov.vn/api/sco-query/invoices/sold?sort=tdlap:desc&size={}&search={}",
            size_page,
            search
        ),
        _ => panic!("Invalid invoice type"),
    }
}

async fn fetch_all_invoices(
    invoice_type: u8,
    from_date: String,
    to_date: String,
    token: Option<String>,
    delay: Option<u64>,
    cancel: Arc<AtomicBool>,
) -> Result<Vec<Value>, String> {
    let mut result = Vec::new();
    let mut current =
        NaiveDate::parse_from_str(&from_date, "%Y-%m-%d").map_err(|e| e.to_string())?;

    let end = NaiveDate::parse_from_str(&to_date, "%Y-%m-%d").map_err(|e| e.to_string())?;

    let invoice_types = match invoice_type {
        1 => vec![1],
        2 => vec![2],
        3 => vec![3],
        4 => vec![4],
        _ => vec![invoice_type],
    };

    while current <= end {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        // ngày cuối của tháng hiện tại
        let first_next_month = if current.month() == 12 {
            NaiveDate::from_ymd_opt(current.year() + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1).unwrap()
        };

        let month_end = (first_next_month - Duration::days(1)).min(end);

        let from = current.format("%d/%m/%Y").to_string();
        let to = month_end.format("%d/%m/%Y").to_string();

        for current_type in invoice_types.clone() {
            let url = get_url(current_type, &from, &to);
            let mut state: Option<String> = None;
            let mut has_more = true;

            while has_more {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }

                let mut request_url = url.clone();

                if let Some(s) = &state {
                    request_url.push_str(&format!("&state={}", s));
                }

                let res: Value =
                    crate::api::http::get(&request_url, token.as_deref(), delay, None, None)
                        .await?;

                if let Some(arr) = res["datas"].as_array() {
                    result.extend(arr.iter().cloned());
                }

                if res["state"].is_null() {
                    has_more = false;
                } else {
                    state = res["state"].as_str().map(|s| s.to_string());
                }

                tokio::time::sleep(std::time::Duration::from_millis(delay.unwrap_or(500))).await;
            }
        }

        // sang tháng tiếp theo
        current = first_next_month;
    }

    result.sort_by(|a, b| {
        let date_a = a["tdlap"].as_str().unwrap_or("");
        let date_b = b["tdlap"].as_str().unwrap_or("");

        match date_a.cmp(date_b) {
            CmpOrdering::Equal => {
                let no_a = a["shdon"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<u64>()
                    .unwrap_or(0);

                let no_b = b["shdon"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<u64>()
                    .unwrap_or(0);

                no_a.cmp(&no_b)
            }
            other => other,
        }
    });

    Ok(result)
}

async fn fetch_invoice_detail(
    datas: Vec<Value>,
    token: Option<String>,
    delay: Option<u64>,
    cancel: Arc<AtomicBool>,
) -> Result<Vec<Value>, String> {
    let mut result = vec![];
    for (_i, item) in datas.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        let nbmst = item["nbmst"].as_str().unwrap_or("");
        let khhdon = item["khhdon"].as_str().unwrap_or("");
        let shdon = item["shdon"].as_i64().unwrap_or(0);
        let khmshdon = item["khmshdon"].as_i64().unwrap_or(0);

        let url = format!(
            "https://hoadondientu.gdt.gov.vn/api/query/invoices/detail?nbmst={}&khhdon={}&shdon={}&khmshdon={}",
            nbmst, khhdon, shdon, khmshdon
        );
        match crate::api::http::get(&url, token.as_deref(), delay, None, None).await {
            Ok(res) => {
                // println!("SUCCESS: {} res={:#?}", url, res);
                result.push(res.clone());
                crate::state::update_sync_emit(|s| {
                    // s.completed = s.completed + 1;
                    s.current_invoice = Some(serde_json::json!(res.clone()));
                    // s.failed = 0;
                });
            }
            Err(e) => {
                println!("ERROR: {} => {}", url, e);
                crate::state::update_sync_emit(|s| {
                    // s.completed = s.completed + 1;
                    s.current_invoice = Some(serde_json::json!(item.clone()));
                    s.failed = s.failed + 1;
                });
                continue;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(delay.unwrap_or(1000))).await;
    }

    Ok(result)
}

pub async fn run_sync_flow(
    invoice_type: u8,
    from_date: String,
    to_date: String,
    token: Option<String>,
    delay: Option<u64>,
) {
    let cancel = crate::state::get_cancel_flag();

    let invoices = match fetch_all_invoices(
        invoice_type,
        from_date,
        to_date,
        token.clone(),
        delay,
        cancel.clone(),
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            println!("ERROR: {}", e);
            crate::state::update_sync_emit(|s| {
                s.running = false;
                s.current_invoice = None;
                s.is_error_api = true;
            });
            // re login
            let _ = navigate_to_route("/login");
            return;
        }
    };

    crate::state::update_sync_emit(|s| {
        s.total = Some(invoices.len());
    });

    // 2. fetch detail
    let _details = match fetch_invoice_detail(invoices, token, delay, cancel).await {
        Ok(v) => v,
        Err(_) => vec![],
    };

    // 3. final state
    crate::state::update_sync_emit(|s| {
        s.running = false;
        s.current_invoice = None;
    });
}
