mod error;
mod model;

pub use error::*;
pub use model::*;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;
pub fn get_printers() -> Result<Vec<PrinterInfo>> {
    #[cfg(target_os = "windows")]
    {
        return windows::get_printers();
    }

    #[cfg(target_os = "macos")]
    {
        return macos::get_printers();
    }

    #[cfg(target_os = "linux")]
    {
        return linux::get_printers();
    }

    #[allow(unreachable_code)]
    Err(PrinterError::Message("Unsupported platform".into()))
}

pub fn print_pdf(options: PrintOptions, pdf_path: &str) -> Result<i32> {
    #[cfg(target_os = "windows")]
    {
        return 0.0;
    }

    #[cfg(target_os = "macos")]
    {
        return macos::print_pdf(options, pdf_path);
    }

    #[cfg(target_os = "linux")]
    {
        return 0.0;
    }

    #[allow(unreachable_code)]
    Err(PrinterError::Message("Unsupported platform".into()))
}

pub fn get_default_printer_id() -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        return "";
    }

    #[cfg(target_os = "macos")]
    {
        return macos::get_default_printer_id();
    }

    #[cfg(target_os = "linux")]
    {
        return "";
    }

    #[allow(unreachable_code)]
    Err(PrinterError::Message("Unsupported platform".into()))
}
