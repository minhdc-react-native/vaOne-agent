use std::fs;

use crate::pdf::models::PdfTemplate;

pub fn load(path: &str) -> anyhow::Result<PdfTemplate> {
    let json = fs::read_to_string(path)?;

    let document: PdfTemplate = serde_json::from_str(&json)?;

    Ok(document)
}
