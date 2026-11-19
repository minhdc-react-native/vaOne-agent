use super::gdi::GdiPrinter;
use super::printer_status::get_printer_status;
use crate::{PDFIUM_PATH, PrintOptions, PrinterError, PrinterInfo, Result};
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
        EnumPrintersW(flags, None, 2, None, &mut needed, &mut returned);

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
    println!("option={:#?}", options);

    let printer_name = options
        .printer
        .or_else(get_default_printer_name)
        .ok_or_else(|| PrinterError::Message("Không tìm thấy máy in".into()))?;

    let status = get_printer_status(&printer_name)?;
    
    println!("status printer={:#?} printer_name={}", status, printer_name);

    // Máy in đang có lỗi
    if status.has_error {
        let message = if !status.messages.is_empty() {
            status.messages.join(", ")
        } else if !status.accepting {
            "Máy in hiện không nhận lệnh in.".to_string()
        } else {
            "Máy in đang gặp lỗi.".to_string()
        };

        return Err(PrinterError::Message(message));
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

    let page_count = document.pages().len() as usize;

    println!("================ PDFIUM ================");
    println!("PDF: {}", pdf_path);
    println!("Pages: {}", page_count);

    if page_count == 0 {
        return Err(PrinterError::Message("PDF không có trang nào.".into()));
    }

    let mut printer = GdiPrinter::new(&printer_name, options.paper.as_deref(), options.duplex)?;

    let (dpi_x, dpi_y) = printer.dpi();

    println!("Printer DPI: {} x {}", dpi_x, dpi_y);

    printer.start_document("vaOne Print")?;

    let selected_pages = parse_page_ranges(options.page_ranges.as_deref(), page_count)?;

    println!(
        "Selected pages: {:?}",
        selected_pages.iter().map(|p| p + 1).collect::<Vec<_>>()
    );

    let copies = options.copies.unwrap_or(1).max(1);

    for copy in 0..copies {
        println!("========== COPY {} / {} ==========", copy + 1, copies);

        for &page_index in &selected_pages {
            println!("========== PAGE {} ==========", page_index + 1);

            let page = document
                .pages()
                .get(page_index as u16)
                .map_err(|e| PrinterError::Message(e.to_string()))?;

            let page_width_pt = page.width().value;
            let page_height_pt = page.height().value;

            println!("PDF size: {:.2} x {:.2} pt", page_width_pt, page_height_pt);

            let width_px = (page_width_pt * dpi_x as f32 / 72.0).round() as i32;

            let height_px = (page_height_pt * dpi_y as f32 / 72.0).round() as i32;

            let bitmap = page
                .render_with_config(
                    &PdfRenderConfig::new()
                        .set_target_width(width_px)
                        .set_target_height(height_px)
                        .set_reverse_byte_order(true),
                )
                .map_err(|e| PrinterError::Message(e.to_string()))?;

            printer.start_page()?;

            printer.print_bitmap(&bitmap)?;

            printer.end_page()?;
        }
    }

    printer.end_document()?;

    println!("========================================");

    Ok(0)
}

fn parse_page_ranges(
    page_ranges: Option<&str>,
    page_count: usize,
) -> Result<Vec<usize>> {
    let Some(page_ranges) = page_ranges else {
        return Ok((0..page_count).collect());
    };

    let mut pages = Vec::new();

    for part in page_ranges.split(',') {
        let part = part.trim();

        if part.is_empty() {
            continue;
        }

        if let Some((start, end)) = part.split_once('-') {
            let start: usize = start
                .trim()
                .parse()
                .map_err(|_| PrinterError::Message(format!("Invalid page range: {}", part)))?;

            let end: usize = end
                .trim()
                .parse()
                .map_err(|_| PrinterError::Message(format!("Invalid page range: {}", part)))?;

            if start == 0 || end == 0 || start > end {
                return Err(PrinterError::Message(format!(
                    "Invalid page range: {}",
                    part
                )));
            }

            if start > page_count {
                continue;
            }

            let end = end.min(page_count);

            for page in start..=end {
                pages.push(page - 1);
            }
        } else {
            let page: usize = part
                .parse()
                .map_err(|_| PrinterError::Message(format!("Invalid page number: {}", part)))?;

            if page == 0 {
                return Err(PrinterError::Message(format!(
                    "Invalid page number: {}",
                    part
                )));
            }

            if page <= page_count {
                pages.push(page - 1);
            }
        }
    }

    pages.sort_unstable();
    pages.dedup();

    Ok(pages)
}
