use chrono::{FixedOffset, NaiveDate, TimeZone};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn get_url(invoice_type: u8, id_account: &str) -> String {
    match invoice_type {
        // 1: Mua vào
        1 | 3 => format!(
            "https://login.saveinvoice.vn/api/invoices/account/{}/type/purchase",
            id_account
        ),
        // 2: Bán ra
        2 | 4 => format!(
            "https://login.saveinvoice.vn/api/invoices/account/{}/type/sold",
            id_account
        ),
        _ => panic!("Invalid invoice type"),
    }
}

fn build_between_value(from_date: &str, to_date: &str) -> Result<String, String> {
    let tz = FixedOffset::east_opt(7 * 3600).unwrap();

    let from = NaiveDate::parse_from_str(from_date, "%Y-%m-%d")
        .map_err(|e| e.to_string())?
        .and_hms_milli_opt(0, 0, 0, 0)
        .unwrap();

    let to = NaiveDate::parse_from_str(to_date, "%Y-%m-%d")
        .map_err(|e| e.to_string())?
        .and_hms_milli_opt(23, 59, 59, 999)
        .unwrap();

    let from_ms = tz
        .from_local_datetime(&from)
        .single()
        .unwrap()
        .timestamp_millis();

    let to_ms = tz
        .from_local_datetime(&to)
        .single()
        .unwrap()
        .timestamp_millis();

    Ok(format!("{}@{}", from_ms, to_ms))
}

async fn fetch_save_invoices(
    invoice_type: u8,
    from_date: String,
    to_date: String,
    token: Option<String>,
    delay: Option<u64>,
    cancel: Arc<AtomicBool>,
    id_account: String,
) -> Result<Vec<Value>, String> {
    let mut result = Vec::new();
    let page_size = 50;
    let mut page = 0;

    let between = build_between_value(&from_date, &to_date)?;

    let mut headers = HashMap::new();
    if let Some(token) = token {
        headers.insert("apiToken".to_string(), token);
    }

    let url = get_url(invoice_type, &id_account);

    loop {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        let lazy_load_event = json!({
            "first": page * page_size,
            "rows": page_size,
            "page": page,
            "sortField": null,
            "sortOrder": null,
            "filters": {
                "tags": {
                    "value": null,
                    "matchMode": "in"
                },
                "data.tdlap": {
                    "value": between,
                    "matchMode": "between"
                }
            }
        });

        let mut params = HashMap::new();
        params.insert("lazyLoadEvent".to_string(), lazy_load_event);

        let res: Value =
            crate::api::http::get(&url, None, delay, Some(headers.clone()), Some(params)).await?;

        let Some(items) = res["items"].as_array() else {
            break;
        };

        if items.is_empty() {
            break;
        }

        result.extend(items.iter().cloned());

        page += 1;
    }

    Ok(result)
}

pub async fn run_sync_flow_save_invoice(
    invoice_type: u8,
    from_date: String,
    to_date: String,
    token: Option<String>,
    delay: Option<u64>,
    id_account: String,
) {
    let cancel = crate::state::get_cancel_flag();

    let invoices = match fetch_save_invoices(
        invoice_type,
        from_date,
        to_date,
        token.clone(),
        delay,
        cancel.clone(),
        id_account,
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
            return;
        }
    };

    crate::state::update_sync_emit(|s| {
        s.total = Some(invoices.len());
    });

    for item in invoices {
        crate::state::update_sync_emit(|s| {
            // s.completed = s.completed + 1;
            s.current_invoice = Some(serde_json::json!(item.clone()));
        });
        tokio::time::sleep(std::time::Duration::from_millis(delay.unwrap_or(1000))).await;
    }

    // 3. final state
    crate::state::update_sync_emit(|s| {
        s.running = false;
        s.current_invoice = None;
    });
}
