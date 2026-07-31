use crate::{PrinterError, Result};

use pdfium_render::prelude::*;

use std::{ffi::c_void, mem::size_of};

use windows::{
    core::PCWSTR,
    Win32::Graphics::Gdi::{
        CreateDCW,
        DeleteDC,
        GetDeviceCaps,
        StretchDIBits,
        BITMAPINFO,
        BITMAPINFOHEADER,
        DIB_RGB_COLORS,
        HDC,
        HORZRES,
        LOGPIXELSX,
        LOGPIXELSY,
        SRCCOPY,
        VERTRES,
    },
};

#[repr(C)]
struct DOCINFOW {
    cbSize: i32,
    lpszDocName: PCWSTR,
    lpszOutput: PCWSTR,
    lpszDatatype: PCWSTR,
    fwType: u32,
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn StartDocW(
        hdc: HDC,
        lpdi: *const DOCINFOW,
    ) -> i32;

    fn StartPage(
        hdc: HDC,
    ) -> i32;

    fn EndPage(
        hdc: HDC,
    ) -> i32;

    fn EndDoc(
        hdc: HDC,
    ) -> i32;
}

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

pub struct GdiPrinter {
    hdc: windows::Win32::Graphics::Gdi::HDC,
    printer_name: String,
    dpi_x: i32,
    dpi_y: i32,
    width: i32,
    height: i32,
    document_started: bool,
    page_started: bool,
}

impl GdiPrinter {
    pub fn new(printer_name: &str) -> Result<Self> {
        unsafe {
            let printer_wide = to_wide(printer_name);
            let driver_wide = to_wide("WINSPOOL");

            let hdc = CreateDCW(
                PCWSTR(driver_wide.as_ptr()),
                PCWSTR(printer_wide.as_ptr()),
                PCWSTR::null(),
                None,
            );

            if hdc.is_invalid() {
                return Err(PrinterError::Message(format!(
                    "Không thể tạo printer DC cho máy in: {}",
                    printer_name
                )));
            }

            let dpi_x = GetDeviceCaps(Some(hdc), LOGPIXELSX);
            let dpi_y = GetDeviceCaps(Some(hdc), LOGPIXELSY);

            let width = GetDeviceCaps(Some(hdc), HORZRES);
            let height = GetDeviceCaps(Some(hdc), VERTRES);

            println!("========== GDI PRINTER ==========");
            println!("Printer : {}", printer_name);
            println!("DPI     : {} x {}", dpi_x, dpi_y);
            println!("Size    : {} x {}", width, height);
            println!("=================================");

            Ok(Self {
                hdc,
                printer_name: printer_name.to_string(),
                dpi_x,
                dpi_y,
                width,
                height,
                document_started: false,
                page_started: false,
            })
        }
    }

    pub fn start_document(&mut self, document_name: &str) -> Result<()> {
        if self.document_started {
            return Err(PrinterError::Message("Tài liệu in đã được bắt đầu.".into()));
        }

        unsafe {
            let document_wide = to_wide(document_name);

            let doc_info = DOCINFOW {
                cbSize: size_of::<DOCINFOW>() as i32,
                lpszDocName: PCWSTR(document_wide.as_ptr()),
                lpszOutput: PCWSTR::null(),
                lpszDatatype: PCWSTR::null(),
                fwType: 0,
            };

            let result = StartDocW(self.hdc, &doc_info);

            if result <= 0 {
                return Err(PrinterError::Message(
                    "Không thể bắt đầu tài liệu in.".into(),
                ));
            }

            self.document_started = true;

            println!("GDI StartDoc: {}", document_name);

            Ok(())
        }
    }

    pub fn start_page(&mut self) -> Result<()> {
        if !self.document_started {
            return Err(PrinterError::Message("Chưa bắt đầu tài liệu in.".into()));
        }

        if self.page_started {
            return Err(PrinterError::Message(
                "Trang in hiện tại chưa được kết thúc.".into(),
            ));
        }

        unsafe {
            let result = StartPage(self.hdc);

            if result <= 0 {
                return Err(PrinterError::Message("Không thể bắt đầu trang in.".into()));
            }

            self.page_started = true;

            Ok(())
        }
    }

    pub fn print_bitmap(&mut self, bitmap: &PdfBitmap) -> Result<()> {
        if !self.page_started {
            return Err(PrinterError::Message("Chưa bắt đầu trang in.".into()));
        }

        let width = bitmap.width();
        let height = bitmap.height();

        if width <= 0 || height <= 0 {
            return Err(PrinterError::Message(
                "Kích thước bitmap không hợp lệ.".into(),
            ));
        }

        // Chuẩn hóa pixel thành RGBA.
        let rgba = bitmap.as_rgba_bytes();

        let expected_size = width as usize * height as usize * 4;

        if rgba.len() < expected_size {
            return Err(PrinterError::Message(format!(
                "Bitmap không hợp lệ: {} bytes, cần {} bytes.",
                rgba.len(),
                expected_size
            )));
        }

        println!(
            "GDI print bitmap: {} x {} ({} bytes)",
            width,
            height,
            rgba.len()
        );

        unsafe {
            let bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: size_of::<BITMAPINFOHEADER>() as u32,

                    // DIB top-down.
                    biWidth: width,
                    biHeight: -height,

                    biPlanes: 1,
                    biBitCount: 32,

                    // BI_RGB = uncompressed.
                    biCompression: 0,

                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },

                bmiColors: [Default::default()],
            };

            /*
             * Pdfium trả RGBA.
             *
             * GDI với 32-bit BI_RGB thực tế mong BGRX/BGRA.
             * Vì vậy cần đổi R <-> B trước khi gửi.
             */
            let mut bgra = rgba;

            for pixel in bgra.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }

            let result = StretchDIBits(
                self.hdc,
                // Destination.
                0,
                0,
                self.width,
                self.height,
                // Source.
                0,
                0,
                width,
                height,
                Some(bgra.as_ptr() as *const c_void),
                &bitmap_info,
                DIB_RGB_COLORS,
                SRCCOPY,
            );

            if result == 0 {
                return Err(PrinterError::Message(
                    "StretchDIBits không thể in bitmap.".into(),
                ));
            }

            println!("StretchDIBits result: {}", result);

            Ok(())
        }
    }

    pub fn end_page(&mut self) -> Result<()> {
        if !self.page_started {
            return Err(PrinterError::Message(
                "Không có trang in đang hoạt động.".into(),
            ));
        }

        unsafe {
            let result = EndPage(self.hdc);

            if result <= 0 {
                return Err(PrinterError::Message("Không thể kết thúc trang in.".into()));
            }

            self.page_started = false;

            Ok(())
        }
    }

    pub fn end_document(&mut self) -> Result<()> {
        if !self.document_started {
            return Err(PrinterError::Message("Chưa bắt đầu tài liệu in.".into()));
        }

        if self.page_started {
            return Err(PrinterError::Message("Trang in chưa được kết thúc.".into()));
        }

        unsafe {
            let result = EndDoc(self.hdc);

            if result <= 0 {
                return Err(PrinterError::Message(
                    "Không thể kết thúc tài liệu in.".into(),
                ));
            }

            self.document_started = false;

            println!("GDI EndDoc: {}", self.printer_name);

            Ok(())
        }
    }

    pub fn dpi(&self) -> (i32, i32) {
        (self.dpi_x, self.dpi_y)
    }

    pub fn printable_size(&self) -> (i32, i32) {
        (self.width, self.height)
    }
}

impl Drop for GdiPrinter {
    fn drop(&mut self) {
        unsafe {
            /*
             * Nếu có lỗi giữa chừng, cố gắng đóng page/doc
             * để không giữ printer DC ở trạng thái dang dở.
             */
            if self.page_started {
                let _ = EndPage(self.hdc);
                self.page_started = false;
            }

            if self.document_started {
                let _ = EndDoc(self.hdc);
                self.document_started = false;
            }

            DeleteDC(self.hdc);
        }
    }
}
