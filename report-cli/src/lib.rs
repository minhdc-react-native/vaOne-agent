use anyhow::{Context, Result};
use std::fs;

use pdf_core::models::PdfTemplate;
use pdf_core::renderer::render_page;

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

#[unsafe(no_mangle)]
pub extern "C" fn render_pdf(
    report_path: *const c_char,
    data_path: *const c_char,
    output_path: *const c_char,
) -> c_int {
    let result = (|| -> anyhow::Result<()> {
        let report = unsafe { CStr::from_ptr(report_path) }.to_str()?;
        let data = unsafe { CStr::from_ptr(data_path) }.to_str()?;
        let output = unsafe { CStr::from_ptr(output_path) }.to_str()?;

        render(report, data, output)
    })();

    if result.is_ok() {
        0
    } else {
        -1
    }
}

pub fn render(report_path: &str, data_path: &str, output_path: &str) -> Result<()> {
    let json_report =
        fs::read_to_string(report_path).with_context(|| format!("Cannot read {}", report_path))?;

    let json_data =
        fs::read_to_string(data_path).with_context(|| format!("Cannot read {}", data_path))?;

    let docs: Vec<PdfTemplate> =
        serde_json::from_str(&json_report).context("Invalid report json")?;

    let datas: Vec<serde_json::Value> =
        serde_json::from_str(&json_data).context("Invalid data json")?;

    let mut progress = |_| {};

    render_page(docs, datas, output_path, &mut progress)?;

    Ok(())
}
