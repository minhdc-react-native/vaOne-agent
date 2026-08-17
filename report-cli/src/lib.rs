use anyhow::{Context, Result};
use std::fs;

use pdf_core::models::PdfTemplate;
use pdf_core::renderer::render_page;
use std::ffi::{c_char, c_int, CStr, CString};
use std::ptr;

use serde_json::Value;

use pdf_core::renderer::render_bytes;

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

/// Callback progress từ Rust -> C#
///
/// C# sẽ nhận một JSON string, ví dụ:
///
/// {"currentReport":1,"totalReport":5}
///
/// hoặc:
///
/// {"message":"Đang lưu file pdf...","current":0}
#[allow(non_camel_case_types)]
pub type ProgressCallback = extern "C" fn(progress_json: *const c_char);

/// Kết quả trả về từ Rust -> C#
#[repr(C)]
pub struct RenderResult {
    /// 1 = thành công
    /// 0 = lỗi
    pub success: i32,

    /// Pointer tới PDF bytes
    pub data: *mut u8,

    /// Số lượng bytes
    pub len: usize,

    /// Error message UTF-8
    ///
    /// NULL nếu thành công
    pub error: *mut c_char,
}

/// Render PDF từ JSON.
///
/// reports_json:
///     JSON array của PdfTemplate
///
/// datas_json:
///     JSON array của data
///
/// progress_callback:
///     callback để Rust gửi progress về C#
#[no_mangle]
pub extern "C" fn render_pdf_to_bytes(
    reports_json: *const c_char,
    datas_json: *const c_char,
    progress_callback: Option<ProgressCallback>,
) -> RenderResult {
    match render_pdf_internal(reports_json, datas_json, progress_callback) {
        Ok(bytes) => {
            let len = bytes.len();

            // Chuyển Vec<u8> sang memory thuộc Rust.
            //
            // C# phải gọi free_render_data()
            // sau khi copy bytes xong.
            let data = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;

            RenderResult {
                success: 1,
                data,
                len,
                error: ptr::null_mut(),
            }
        }

        Err(e) => {
            let error = CString::new(e.to_string())
                .unwrap_or_else(|_| CString::new("Unknown render error").unwrap())
                .into_raw();

            RenderResult {
                success: 0,
                data: ptr::null_mut(),
                len: 0,
                error,
            }
        }
    }
}

fn render_pdf_internal(
    reports_json: *const c_char,
    datas_json: *const c_char,
    progress_callback: Option<ProgressCallback>,
) -> Result<Vec<u8>> {
    if reports_json.is_null() {
        anyhow::bail!("reports_json is null");
    }

    if datas_json.is_null() {
        anyhow::bail!("datas_json is null");
    }

    // C char* -> Rust &str
    let reports_json = unsafe { CStr::from_ptr(reports_json) }
        .to_str()
        .context("reports_json is not valid UTF-8")?;

    let datas_json = unsafe { CStr::from_ptr(datas_json) }
        .to_str()
        .context("datas_json is not valid UTF-8")?;

    // JSON -> Vec<PdfTemplate>
    let docs: Vec<PdfTemplate> =
        serde_json::from_str(reports_json).context("Invalid reports JSON")?;

    // JSON -> Vec<Value>
    let datas: Vec<Value> = serde_json::from_str(datas_json).context("Invalid datas JSON")?;

    // Callback progress
    let mut progress = |value: Value| {
        if let Some(callback) = progress_callback {
            send_progress(callback, &value);
        }
    };

    // Gọi trực tiếp hàm render_bytes hiện tại
    render_bytes(docs, datas, &mut progress)
}

/// Gửi progress từ Rust sang C#
fn send_progress(callback: ProgressCallback, value: &Value) {
    let json = match serde_json::to_string(value) {
        Ok(json) => json,

        Err(_) => return,
    };

    let c_json = match CString::new(json) {
        Ok(value) => value,

        Err(_) => return,
    };

    callback(c_json.as_ptr());
}

/// Giải phóng PDF bytes được Rust trả về.
///
/// C# phải gọi hàm này sau khi Marshal.Copy()
#[no_mangle]
pub extern "C" fn free_render_data(data: *mut u8, len: usize) {
    if data.is_null() || len == 0 {
        return;
    }

    unsafe {
        let slice = std::slice::from_raw_parts_mut(data, len);

        let _ = Box::from_raw(slice as *mut [u8]);
    }
}

/// Giải phóng error string.
#[no_mangle]
pub extern "C" fn free_render_error(error: *mut c_char) {
    if error.is_null() {
        return;
    }

    unsafe {
        let _ = CString::from_raw(error);
    }
}
