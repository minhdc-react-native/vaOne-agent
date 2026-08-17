use std::{path::PathBuf, sync::OnceLock};

pub static PDFIUM_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init_pdfium(path: PathBuf) {
    PDFIUM_PATH.set(path).ok();
}
