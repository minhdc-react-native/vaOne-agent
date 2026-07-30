pub mod printer;
pub mod printer_status;
pub mod gdi;
pub use printer::{get_printers, print_pdf};
pub use printer_status::get_printer_status;
