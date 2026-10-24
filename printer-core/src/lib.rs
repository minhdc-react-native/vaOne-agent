mod error;
mod model;
mod state;
pub use error::*;
pub use model::*;
pub use state::init_pdfium;
pub use state::*;

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;
pub fn get_printers() -> Result<Vec<PrinterInfo>> {
    #[cfg(target_os = "windows")]
    {
        return win::get_printers();
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
        return win::print_pdf(options, pdf_path);
    }

    #[cfg(target_os = "macos")]
    {
        return macos::print_pdf(options, pdf_path);
    }

    #[cfg(target_os = "linux")]
    {
        return linux::print_pdf(options, pdf_path);
    }

    #[allow(unreachable_code)]
    Err(PrinterError::Message("Unsupported platform".into()))
}
