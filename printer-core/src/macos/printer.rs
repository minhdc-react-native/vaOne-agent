use crate::{PrintOptions, PrinterError, PrinterInfo, PrinterStatus, Result};
use libc::{c_char, c_int, c_void};
use std::ffi::{CStr, CString};

#[repr(C)]
pub struct cups_option_t {
    pub name: *const c_char,
    pub value: *const c_char,
}

#[repr(C)]
pub struct cups_dest_t {
    pub name: *const c_char,
    pub instance: *const c_char,
    pub is_default: c_int,
    pub num_options: c_int,
    pub options: *mut cups_option_t,
}

#[link(name = "cups")]
unsafe extern "C" {
    fn cupsLastErrorString() -> *const c_char;

    fn cupsGetDests(dests: *mut *mut cups_dest_t) -> c_int;

    fn cupsFreeDests(num_dests: c_int, dests: *mut cups_dest_t);

    fn cupsGetOption(
        name: *const c_char,
        num_options: c_int,
        options: *const cups_option_t,
    ) -> *const c_char;

    fn cupsGetNamedDest(
        http: *mut c_void,
        name: *const c_char,
        instance: *const c_char,
    ) -> *mut cups_dest_t;

    fn cupsPrintFile(
        printer: *const c_char,
        filename: *const c_char,
        title: *const c_char,
        num_options: c_int,
        options: *const cups_option_t,
    ) -> c_int;
}

pub fn get_printers() -> Result<Vec<PrinterInfo>> {
    unsafe {
        let mut dests: *mut cups_dest_t = std::ptr::null_mut();

        let count = cupsGetDests(&mut dests);

        let mut printers = Vec::with_capacity(count as usize);

        for i in 0..count {
            let dest = &*dests.add(i as usize);

            let id = if dest.name.is_null() {
                String::new()
            } else {
                std::ffi::CStr::from_ptr(dest.name)
                    .to_string_lossy()
                    .into_owned()
            };

            let display_name = get_option(dest, "printer-info").unwrap_or_else(|| id.clone());

            printers.push(PrinterInfo {
                id,
                name: display_name,
                model: get_option(dest, "printer-make-and-model"),
                location: get_option(dest, "printer-location"),
                uri: get_option(dest, "device-uri"),
                is_default: dest.is_default != 0,
            });
        }

        cupsFreeDests(count, dests);

        Ok(printers)
    }
}

fn get_default_printer_id() -> Result<String> {
    get_printers()?
        .into_iter()
        .find(|p| p.is_default)
        .map(|p| p.id)
        .ok_or_else(|| PrinterError::Message("Default printer not found".into()))
}

fn get_option(dest: &cups_dest_t, key: &str) -> Option<String> {
    unsafe {
        for i in 0..dest.num_options {
            let option = &*dest.options.add(i as usize);

            if option.name.is_null() {
                continue;
            }

            let name = std::ffi::CStr::from_ptr(option.name).to_string_lossy();

            if name == key {
                if option.value.is_null() {
                    return None;
                }

                return Some(
                    std::ffi::CStr::from_ptr(option.value)
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }

        None
    }
}

pub fn print_pdf(options: PrintOptions, pdf_path: &str) -> Result<i32> {
    unsafe {
        let printer_name = match options.printer {
            Some(printer) => printer,
            None => get_default_printer_id()?,
        };
        let status = get_printer_status(&printer_name)?;
        println!("status printer={:#?}", status);
        if status.has_error {
            return Err(PrinterError::Message(if !status.messages.is_empty() {
                status.messages.join(", ")
            } else if !status.accepting {
                "Máy in hiện không nhận lệnh in.".into()
            } else if status.state == 5 {
                "Máy in đang tạm dừng.".into()
            } else {
                "Máy in đang gặp lỗi.".into()
            }));
        }

        let printer =
            CString::new(printer_name).map_err(|e| PrinterError::Message(e.to_string()))?;

        let file = CString::new(pdf_path).map_err(|e| PrinterError::Message(e.to_string()))?;

        let title = CString::new("Print Job").unwrap();

        // =========================
        // Build CUPS options
        // =========================

        let mut option_names: Vec<CString> = Vec::new();
        let mut option_values: Vec<CString> = Vec::new();

        // copies
        if let Some(copies) = options.copies {
            option_names.push(CString::new("copies").unwrap());

            option_values.push(CString::new(copies.to_string()).unwrap());
        }

        // duplex
        if let Some(duplex) = options.duplex {
            option_names.push(CString::new("sides").unwrap());

            option_values.push(
                CString::new(if duplex {
                    "two-sided-long-edge"
                } else {
                    "one-sided"
                })
                .unwrap(),
            );
        }

        // color
        if let Some(color) = options.color {
            option_names.push(CString::new("print-color-mode").unwrap());

            option_values.push(CString::new(if color { "color" } else { "monochrome" }).unwrap());
        }

        // paper
        if let Some(paper) = options.paper {
            option_names.push(CString::new("media").unwrap());

            option_values.push(CString::new(paper).unwrap());
        }

        // landscape
        if let Some(landscape) = options.landscape {
            option_names.push(CString::new("orientation-requested").unwrap());

            option_values.push(
                CString::new(if landscape {
                    "4" // landscape
                } else {
                    "3" // portrait
                })
                .unwrap(),
            );
        }

        // page ranges
        if let Some(page_ranges) = options.page_ranges {
            option_names.push(CString::new("page-ranges").unwrap());

            option_values.push(CString::new(page_ranges).unwrap());
        }

        // Convert sang cups_option_t
        let mut cups_options: Vec<cups_option_t> = Vec::new();

        for i in 0..option_names.len() {
            cups_options.push(cups_option_t {
                name: option_names[i].as_ptr(),
                value: option_values[i].as_ptr(),
            });
        }

        let job_id = cupsPrintFile(
            printer.as_ptr(),
            file.as_ptr(),
            title.as_ptr(),
            cups_options.len() as i32,
            cups_options.as_ptr(),
        );

        if job_id == 0 {
            let err = CStr::from_ptr(cupsLastErrorString())
                .to_string_lossy()
                .into_owned();

            return Err(PrinterError::Message(err));
        }

        Ok(job_id)
    }
}

pub fn get_printer_status(printer: &str) -> Result<PrinterStatus> {
    unsafe {
        let printer = CString::new(printer).map_err(|e| PrinterError::Message(e.to_string()))?;

        let dest = cupsGetNamedDest(std::ptr::null_mut(), printer.as_ptr(), std::ptr::null());

        if dest.is_null() {
            return Err(PrinterError::Message("Không tìm thấy máy in.".into()));
        }

        let dest = &*dest;

        let get = |name: &str| -> Option<String> {
            let key = CString::new(name).unwrap();

            let value = cupsGetOption(key.as_ptr(), dest.num_options, dest.options);

            if value.is_null() {
                None
            } else {
                Some(CStr::from_ptr(value).to_string_lossy().into_owned())
            }
        };

        let state = get("printer-state")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let reasons: Vec<String> = get("printer-state-reasons")
            .unwrap_or_default()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let accepting = get("printer-is-accepting-jobs")
            .map(|v| v == "true")
            .unwrap_or(true);

        let error_reasons: Vec<&str> = reasons
            .iter()
            .map(String::as_str)
            .filter(|reason| is_error_reason(reason))
            .collect();

        let has_error = !accepting || state == 5 || !error_reasons.is_empty();

        let messages: Vec<String> = error_reasons
            .iter()
            .map(|reason| translate_reason(reason).to_string())
            .collect();

        Ok(PrinterStatus {
            state,
            reasons,
            messages,
            accepting,
            has_error,
        })
    }
}

fn translate_reason(reason: &str) -> &'static str {
    match reason {
        "offline" => "Máy in đang ngoại tuyến",

        "media-empty" | "media-empty-report" => "Máy in cần giấy",

        "media-jam" | "media-jam-report" => "Máy in bị kẹt giấy",

        "door-open" | "door-open-report" => "Nắp máy in đang mở",

        "paused" => "Máy in đang tạm dừng",

        "toner-low" => "Mực in sắp hết",

        "toner-empty" | "marker-supply-empty" | "marker-supply-empty-report" => "Máy in đã hết mực",

        "none" => "Máy in sẵn sàng",

        _ => "Không xác định",
    }
}

fn is_error_reason(reason: &str) -> bool {
    matches!(
        reason,
        "offline"
            | "media-empty"
            | "media-empty-report"
            | "media-jam"
            | "media-jam-report"
            | "door-open"
            | "door-open-report"
            | "paused"
            | "toner-empty"
            | "marker-supply-empty"
            | "marker-supply-empty-report"
    )
}
