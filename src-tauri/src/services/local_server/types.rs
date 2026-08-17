use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
pub struct OpenTrayRequest {
    pub route: String,
    pub data: serde_json::Value,
}

#[derive(Serialize)]
pub struct PingResponse {
    pub success: bool,
}

#[derive(Deserialize)]
pub struct MessageRequest {
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SourceInvoice {
    #[serde(rename = "TCT")]
    Tct,

    #[serde(rename = "M-SMI")]
    M_Smi,

    #[serde(rename = "SAVE-INVOICE")]
    SaveInvoice,

    #[serde(rename = "SHOPEE")]
    Shopee,

    #[serde(rename = "LAZADA")]
    Lazada,

    #[serde(rename = "TIKTOK")]
    Tiktok,
}
