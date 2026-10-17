use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaxSyncError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON processing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid date: {0}")]
    InvalidDate(String),

    #[error("Invalid invoice type: {0}")]
    InvalidInvoiceType(u8),

    #[error("Missing required invoice field: {0}")]
    MissingField(&'static str),

    #[error("Invalid invoice data: {0}")]
    InvalidInvoice(String),

    #[error("Invoice sync was cancelled")]
    Cancelled,

    #[error("Invoice upload failed: {0}")]
    Upload(String),

    #[error("Database operation failed: {0}")]
    Database(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Tax service returned an error: {0}")]
    Api(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Date range is invalid: {0}")]
    InvalidDateRange(String),
}

pub type Result<T> = std::result::Result<T, TaxSyncError>;
