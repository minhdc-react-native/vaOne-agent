use crate::{PrintOptions, PrinterError, PrinterInfo, Result};
use libc::{c_char, c_int};
use std::{
    ffi::{CStr, CString},
    ptr,
};

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
    fn cupsGetDests(dests: *mut *mut cups_dest_t) -> c_int;

    fn cupsFreeDests(num_dests: c_int, dests: *mut cups_dest_t);
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

pub fn get_default_printer_id() -> Result<String> {
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

#[link(name = "cups")]
unsafe extern "C" {
    fn cupsLastErrorString() -> *const c_char;
}

#[link(name = "cups")]
unsafe extern "C" {
    fn cupsPrintFile(
        printer: *const c_char,
        filename: *const c_char,
        title: *const c_char,
        num_options: c_int,
        options: *const cups_option_t,
    ) -> c_int;
}

pub fn print_pdf(options: PrintOptions, pdf_path: &str) -> Result<i32> {
    unsafe {
        let printer_name = match options.printer {
            Some(printer) => printer,
            None => get_default_printer_id()?,
        };

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
