use chrono::NaiveDate;
use serde_json::{json, Value};
use std::cmp::Ordering as CmpOrdering;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::progress_bar;
use crate::services::update::update_progress;

fn format_date(date: &str) -> String {
    NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map(|d| d.format("%d/%m/%Y").to_string())
        .unwrap_or_else(|_| date.to_string()) // nếu parse lỗi thì trả về chuỗi gốc
}

async fn fetch_m_invoices(
    invoice_type: u8,
    from_date: String,
    to_date: String,
    token: Option<String>,
    delay: Option<u64>,
    cancel: Arc<AtomicBool>,
    tax_code: String,
) -> Result<Vec<Value>, String> {
    let mut result = Vec::new();
    let page_size: i32 = 50;
    let mut current_page = 1;
    let mut headers = HashMap::new();
    if let Some(token) = token {
        headers.insert("apiToken".to_string(), token);
    }

    let mut map: HashMap<String, Value> = HashMap::new();

    map.insert("size".to_string(), json!(page_size));
    map.insert(
        "invoiceType".to_string(),
        json!(if invoice_type == 1 {
            "INPUT_ELECTRONIC_INVOICE"
        } else {
            "OUTPUT_ELECTRONIC_INVOICE"
        }),
    );
    map.insert(
        "invoiceReleaseDateFrom".to_string(),
        json!(format_date(&from_date)),
    );
    map.insert(
        "invoiceReleaseDateTo".to_string(),
        json!(format_date(&to_date)),
    );

    if invoice_type == 1 || invoice_type == 3 {
        map.insert("buyerTaxNo".to_string(), json!(tax_code));
    } else {
        map.insert("sellerTaxNo".to_string(), json!(tax_code));
    }

    if invoice_type == 3 || invoice_type == 4 {
        map.insert("khhdon".to_string(), json!("M"));
    }

    let url = format!("https://qlhd.minvoice.com.vn/api/qlhd-api/invoices");

    loop {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        map.insert("page".to_string(), json!(current_page));

        let param: Option<HashMap<String, Value>> = Some(map.clone());

        let res: Value =
            crate::api::http::get(&url, None, delay, Some(headers.clone()), param).await?;

        let Some(items) = res["listInvoice"].as_array() else {
            break;
        };

        if items.is_empty() {
            break;
        }

        result.extend(items.iter().cloned());

        current_page += 1;
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

pub async fn run_sync_flow_m_invoice(
    tenant_id: String,
    org_unit_id: String,
    invoice_type: u8,
    from_date: String,
    to_date: String,
    token: Option<String>,
    delay: Option<u64>,
    tax_code: String,
) {
    let cancel = crate::state::get_cancel_flag();

    let invoices = match fetch_m_invoices(
        invoice_type,
        from_date,
        to_date,
        token.clone(),
        delay,
        cancel.clone(),
        tax_code,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => {
            println!("ERROR: {}", e);
            progress_bar(None, None);
            update_progress(None);
            crate::state::update_sync_emit(&tenant_id, |s| {
                s.running = false;
                s.current_invoice = None;
                s.is_error_api = true;
            });
            return;
        }
    };
    let num_of_invoice: usize = invoices.len();
    let mut completed: usize = 0;
    progress_bar(Some(completed), Some(num_of_invoice));
    let payload = json!({
        "total":Some(num_of_invoice)
    });
    update_progress(Some(&payload));
    crate::state::update_sync_emit(&tenant_id, |s| {
        s.total = Some(num_of_invoice);
    });

    for item in invoices {
        let tdlap = item["tdlap"].as_str().unwrap_or("");
        let khhdon = item["khhdon"].as_str().unwrap_or("");
        let shdon = item["shdon"].as_i64().unwrap_or(0);

        match crate::api::http::post_data(&tenant_id, &org_unit_id, &item).await {
            Ok(_) => {
                completed += 1;
                progress_bar(Some(completed), Some(num_of_invoice));
                let payload = json!({
                    "completed":Some(completed),
                    "invoice":{
                        "invoiceDate": tdlap,
                        "invoiceNumber": shdon,
                        "invoiceSerial": khhdon
                    }
                });
                update_progress(Some(&payload));
                crate::state::update_sync_emit(&tenant_id, |s| {
                    s.completed += 1;
                    s.success += 1;
                    s.current_invoice = Some(item.clone());
                });
            }
            Err(err) => {
                eprintln!("Post invoice failed: {}", err);
                completed += 1;
                progress_bar(Some(completed), Some(num_of_invoice));
                let payload = json!({
                    "completed":Some(completed),
                    "invoice":{
                        "invoiceDate": tdlap,
                        "invoiceNumber": shdon,
                        "invoiceSerial": khhdon
                    }
                });
                update_progress(Some(&payload));
                crate::state::update_sync_emit(&tenant_id, |s| {
                    s.completed += 1;
                    s.failed += 1;
                    s.current_invoice = Some(item.clone());
                    s.message = err;
                    s.is_error_api = true;
                });
            }
        }
    }

    // 3. final state
    progress_bar(None, None);
    update_progress(None);
    crate::state::update_sync_emit(&tenant_id, |s| {
        s.running = false;
        s.current_invoice = None;
    });
}
