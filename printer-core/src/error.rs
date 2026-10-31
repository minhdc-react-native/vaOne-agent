use thiserror::Error;

#[derive(Debug, Error)]
pub enum PrinterError {
    #[error("{0}")]
    Message(String),
}

pub type Result<T> = std::result::Result<T, PrinterError>;
