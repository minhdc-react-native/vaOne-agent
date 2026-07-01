use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
async fn fetch_all_invoices(
    url: String,
    token: Option<String>,
    delay: Option<u64>,
    cancel: Arc<AtomicBool>,
) -> Result<Vec<Value>, String> {
    let mut result = vec![];
    let mut has_more = true;
    let mut fix_url = url.clone();

    while has_more {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        let res: Value = crate::api::http::get(&fix_url, token.as_deref(), delay, None).await?;

        let datas = res["datas"].clone();
        let state = res["state"].clone();

        if let Some(arr) = datas.as_array() {
            result.extend(arr.clone());
        }

        has_more = !state.is_null();
        fix_url = format!("{}&state={}", url, state);

        tokio::time::sleep(std::time::Duration::from_millis(delay.unwrap_or(500))).await;
    }
    Ok(result)
}

async fn fetch_invoice_detail(
    datas: Vec<Value>,
    token: Option<String>,
    delay: Option<u64>,
    cancel: Arc<AtomicBool>,
) -> Result<Vec<Value>, String> {
    let mut result = vec![];

    for (i, item) in datas.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }

        let nbmst = item["nbmst"].as_str().unwrap_or("");
        let khhdon = item["khhdon"].as_str().unwrap_or("");
        let shdon = item["shdon"].as_str().unwrap_or("");
        let khmshdon = item["khmshdon"].as_str().unwrap_or("");

        let url = format!(
            "https://hoadondientu.gdt.gov.vn/api/query/invoices/detail?nbmst={}&khhdon={}&shdon={}&khmshdon={}",
            nbmst, khhdon, shdon, khmshdon
        );
        match crate::api::http::get(&url, token.as_deref(), delay, None).await {
            Ok(res) => {
                result.push(res.clone());
                crate::state::update_sync_emit(|s| {
                    s.completed = s.completed + 1;
                    s.current_invoice = Some(serde_json::json!(item.clone()));
                    s.failed = 0;
                });
            }
            Err(e) => {
                crate::state::update_sync_emit(|s| {
                    s.completed = s.completed + 1;
                    s.current_invoice = Some(serde_json::json!(item.clone()));
                    s.failed = 1;
                });
                continue;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(delay.unwrap_or(1000))).await;
    }

    Ok(result)
}

pub async fn run_sync_flow(url: String, token: Option<String>, delay: Option<u64>) {
    let cancel = crate::state::get_cancel_flag();

    let invoices = match fetch_all_invoices(url, token.clone(), delay, cancel.clone()).await {
        Ok(v) => v,
        Err(e) => {
            crate::state::update_sync_emit(|s| {
                s.source = "TCT".to_string();
                s.running = false;
                s.current_invoice = None;
            });
            return;
        }
    };

    crate::state::update_sync_emit(|s| {
        s.source = "TCT".to_string();
        s.running = true;
        s.total = invoices.len();
        s.completed = 0;
        s.current_invoice = None;
    });

    // 2. fetch detail
    let details = match fetch_invoice_detail(invoices, token, delay, cancel).await {
        Ok(v) => v,
        Err(_) => vec![],
    };

    // 3. final state
    crate::state::update_sync_emit(|s| {
        s.source = "TCT".to_string();
        s.running = false;
        s.current_invoice = None;
    });
}
