use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterInfo {
    pub id: String,

    /// Tên hiển thị
    pub name: String,

    pub model: Option<String>,

    pub location: Option<String>,

    pub uri: Option<String>,

    pub is_default: bool,
}

#[derive(Debug, Deserialize)]
pub struct PrintOptions {
    /// None = máy in mặc định
    pub printer: Option<String>,
    pub copies: Option<u32>,
    pub duplex: Option<bool>,
    pub color: Option<bool>,
    pub paper: Option<String>,
    pub landscape: Option<bool>,
    pub page_ranges: Option<String>,
}

#[derive(Debug)]
pub struct PrinterStatus {
    pub state: u32,
    pub reasons: Vec<String>,
    pub messages: Vec<String>,
    pub accepting: bool,
}
