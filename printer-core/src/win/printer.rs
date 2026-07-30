use crate::{PDFIUM_PATH,PrintOptions,PrinterError, PrinterInfo, Result};
use super::printer_status::get_printer_status;

use pdfium_render::prelude::*;
use std::ptr::null_mut;

use windows::{
    Win32::Graphics::Printing::{
        EnumPrintersW, GetDefaultPrinterW, PRINTER_ENUM_CONNECTIONS, PRINTER_ENUM_LOCAL,
        PRINTER_INFO_2W,
    },
    core::PWSTR,
};
fn pwstr_to_string(ptr: PWSTR) -> String {
    if ptr.is_null() {
        return String::new();
    }

    unsafe {
        let mut len = 0;

        while *ptr.0.add(len) != 0 {
            len += 1;
        }

        String::from_utf16_lossy(std::slice::from_raw_parts(ptr.0, len))
    }
}

fn get_default_printer_name() -> Option<String> {
    unsafe {
        let mut needed = 0;

        let _ = GetDefaultPrinterW(None, &mut needed);

        if needed == 0 {
            return None;
        }

        let mut buffer = vec![0u16; needed as usize];

        if GetDefaultPrinterW(Some(PWSTR(buffer.as_mut_ptr())), &mut needed).as_bool() {
            Some(String::from_utf16_lossy(
                &buffer[..needed.saturating_sub(1) as usize],
            ))
        } else {
            None
        }
    }
}

pub fn get_printers() -> Result<Vec<PrinterInfo>> {
    unsafe {
        let flags = PRINTER_ENUM_LOCAL | PRINTER_ENUM_CONNECTIONS;

        let mut needed = 0u32;
        let mut returned = 0u32;

        // lấy kích thước buffer
        EnumPrintersW(
            flags,
            None,
            2,
            None,
            &mut needed,
            &mut returned,
        );

        if needed == 0 {
            return Ok(Vec::new());
        }

        let mut buffer = vec![0u8; needed as usize];

        EnumPrintersW(
            flags,
            None,
            2,
            Some(buffer.as_mut_slice()),
            &mut needed,
            &mut returned,
        );
        let printers = std::slice::from_raw_parts(
            buffer.as_ptr() as *const PRINTER_INFO_2W,
            returned as usize,
        );

        let default_name = get_default_printer_name();

        let mut result = Vec::with_capacity(printers.len());

        for p in printers {
            let name = pwstr_to_string(PWSTR(p.pPrinterName.0));

            result.push(PrinterInfo {
                id: name.clone(),
                name: name.clone(),
                model: Some(pwstr_to_string(PWSTR(p.pDriverName.0))),
                location: Some(pwstr_to_string(PWSTR(p.pLocation.0))),
                uri: None,
                is_default: default_name.as_ref().map(|d| d == &name).unwrap_or(false),
            });
        }

        Ok(result)
    }
}

pub fn print_pdf(options: PrintOptions, pdf_path: &str) -> Result<i32> {

    let printer_name = options
        .printer
        .or_else(get_default_printer_name)
        .ok_or_else(|| PrinterError::Message("Không tìm thấy máy in".into()))?;

    let status = get_printer_status(&printer_name)?;
    println!("status printer={:#?} printer_name={}", status,printer_name);
    // Máy in không nhận job mới
    if !status.accepting {
        return Err(PrinterError::Message(
            "Máy in hiện không nhận lệnh in.".into(),
        ));
    }

    // Có lý do lỗi
    if !status.messages.is_empty() {
        return Err(PrinterError::Message(status.messages.join(", ")));
    }

    // Printer bị stop
    if status.state == 5 {
        return Err(PrinterError::Message("Máy in đang tạm dừng.".into()));
    }

    let pdfium_path = PDFIUM_PATH.get().ok_or_else(|| {
        PrinterError::Message("PDFium chưa được khởi tạo. Hãy gọi init_pdfium() trước.".into())
    })?;

    let bindings =
        Pdfium::bind_to_library(pdfium_path).map_err(|e| PrinterError::Message(e.to_string()))?;

    let pdfium = Pdfium::new(bindings);

    let document = pdfium
        .load_pdf_from_file(pdf_path, None)
        .map_err(|e| PrinterError::Message(e.to_string()))?;

    let page_count = document.pages().len();

    println!("================ PDFIUM ================");
    println!("PDF: {}", pdf_path);
    println!("Pages: {}", page_count);

    if page_count > 0 {
        let page = document
            .pages()
            .get(0)
            .map_err(|e| PrinterError::Message(e.to_string()))?;

        let width_px = (page.width().value * 300.0 / 72.0).round() as i32;
        let height_px = (page.height().value * 300.0 / 72.0).round() as i32;

        let bitmap = page.render_with_config(
            &PdfRenderConfig::new()
                .set_target_width(width_px)
                .set_target_height(height_px),
        ).map_err(|e| PrinterError::Message(e.to_string()))?;

        println!("Page 1 rendered: {} x {}", bitmap.width(), bitmap.height());
    }

    println!("========================================");

    Ok(0)
}
