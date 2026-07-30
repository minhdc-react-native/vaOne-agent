use std::{
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use pdfium_render::prelude::*;

use crate::{PrinterError, Result};

pub static PDFIUM_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init_pdfium(path: PathBuf) {
    PDFIUM_PATH.set(path).ok();
}
