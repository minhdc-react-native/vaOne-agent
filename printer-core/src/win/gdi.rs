use crate::{PrinterError, Result};

use pdfium_render::prelude::*;

use std::{ffi::c_void, mem::size_of};

use windows::{
    Win32::Graphics::Gdi::{
        BITMAPINFO, BITMAPINFOHEADER, CreateDCW, DEVMODEW, DIB_RGB_COLORS, DM_DUPLEX,
        DMDUP_SIMPLEX, DMDUP_VERTICAL, DeleteDC, GetDeviceCaps, HDC, HORZRES, LOGPIXELSX,
        LOGPIXELSY, PHYSICALHEIGHT, PHYSICALOFFSETX, PHYSICALOFFSETY, PHYSICALWIDTH, SRCCOPY,
        StretchDIBits, VERTRES,
    },
    Win32::Graphics::Printing::{
        ClosePrinter, DM_OUT_BUFFER, DocumentPropertiesW, OpenPrinterW, PRINTER_ACCESS_USE,
        PRINTER_DEFAULTSW,
    },
    core::PCWSTR,
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
    fn StartDocW(hdc: HDC, lpdi: *const DOCINFOW) -> i32;

    fn StartPage(hdc: HDC) -> i32;

    fn EndPage(hdc: HDC) -> i32;

    fn EndDoc(hdc: HDC) -> i32;
}

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

pub struct GdiPrinter {
    hdc: HDC,
    printer_name: String,

    dpi_x: i32,
    dpi_y: i32,

    // Printable area.
    width: i32,
    height: i32,

    // Physical paper.
    physical_width: i32,
    physical_height: i32,

    // Non-printable offset.
    offset_x: i32,
    offset_y: i32,

    document_started: bool,
    page_started: bool,
}

impl GdiPrinter {
    pub fn new(printer_name: &str, duplex: Option<bool>) -> Result<Self> {
        unsafe {
            let printer_wide = to_wide(printer_name);
            let driver_wide = to_wide("WINSPOOL");

            let mut devmode = get_printer_devmode(PCWSTR(printer_wide.as_ptr()))?;

            let devmode_ptr = devmode.as_mut_ptr() as *mut DEVMODEW;

            // Duplex
            if let Some(duplex) = duplex {
                let devmode_ref = &mut *devmode_ptr;

                if devmode_ref.dmFields & DM_DUPLEX == 0 {
                    return Err(PrinterError::Message(
                        "Máy in không hỗ trợ in hai mặt.".into(),
                    ));
                }

                devmode_ref.dmFields |= DM_DUPLEX;

                devmode_ref.dmDuplex = if duplex {
                    DMDUP_VERTICAL
                } else {
                    DMDUP_SIMPLEX
                };
            }

            let hdc = CreateDCW(
                PCWSTR(driver_wide.as_ptr()),
                PCWSTR(printer_wide.as_ptr()),
                PCWSTR::null(),
                Some(&*devmode_ptr),
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

            let physical_width = GetDeviceCaps(Some(hdc), PHYSICALWIDTH);

            let physical_height = GetDeviceCaps(Some(hdc), PHYSICALHEIGHT);

            let offset_x = GetDeviceCaps(Some(hdc), PHYSICALOFFSETX);

            let offset_y = GetDeviceCaps(Some(hdc), PHYSICALOFFSETY);

            println!("========== GDI PRINTER ==========");
            println!("Printer       : {}", printer_name);
            println!("DPI           : {} x {}", dpi_x, dpi_y);
            println!("Printable     : {} x {}", width, height);
            println!("Physical      : {} x {}", physical_width, physical_height);
            println!("Offset        : {} x {}", offset_x, offset_y);
            println!("Duplex        : {:?}", duplex);
            println!("=================================");

            Ok(Self {
                hdc,
                printer_name: printer_name.to_string(),

                dpi_x,
                dpi_y,

                width,
                height,

                physical_width,
                physical_height,

                offset_x,
                offset_y,

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

                    // Top-down DIB.
                    biWidth: width,
                    biHeight: -height,

                    biPlanes: 1,
                    biBitCount: 32,

                    // BI_RGB.
                    biCompression: 0,

                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },

                bmiColors: [Default::default()],
            };

            // PDFium: RGBA
            // GDI 32-bit BI_RGB: BGRA/BGRX
            let mut bgra = rgba;

            for pixel in bgra.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }

            /*
             * Bitmap đã được render theo DPI của printer.
             *
             * Vì vậy KHÔNG scale bitmap lên printable area.
             *
             * Ví dụ:
             *
             * PDF A5 @ 300 DPI
             * 1748 x 2480
             *
             * Printer A4 @ 300 DPI
             * 2480 x 3508
             *
             * => destination vẫn là 1748 x 2480.
             */

            let dest_width = width;
            let dest_height = height;

            /*
             * Tính vị trí theo physical paper.
             *
             * Sau đó chuyển từ physical coordinate
             * sang printable-area coordinate bằng offset.
             */
            // let dest_x =
            //     (self.physical_width - dest_width) / 2
            //         - self.offset_x;

            // let dest_y =
            //     (self.physical_height - dest_height) / 2
            //         - self.offset_y;

            let dest_x = (self.width - dest_width) / 2;
            let dest_y = 0;

            println!(
                "Physical paper: {} x {}",
                self.physical_width, self.physical_height
            );

            println!("Printable area: {} x {}", self.width, self.height);

            println!("Offset: {} x {}", self.offset_x, self.offset_y);

            println!(
                "Print position: x={}, y={}, size={}x{}",
                dest_x, dest_y, dest_width, dest_height
            );

            let result = StretchDIBits(
                self.hdc,
                // Destination.
                dest_x,
                dest_y,
                dest_width,
                dest_height,
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

    pub fn physical_size(&self) -> (i32, i32) {
        (self.physical_width, self.physical_height)
    }

    pub fn physical_offset(&self) -> (i32, i32) {
        (self.offset_x, self.offset_y)
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

unsafe fn get_printer_devmode(printer_name: PCWSTR) -> Result<Vec<u8>> {
    let mut printer_handle = Default::default();

    let defaults = PRINTER_DEFAULTSW {
        pDatatype: PWSTR::null(),
        pDevMode: std::ptr::null_mut(),
        DesiredAccess: PRINTER_ACCESS_USE,
    };

    OpenPrinterW(printer_name, &mut printer_handle, Some(&defaults))
        .ok()
        .map_err(|e| PrinterError::Message(format!("Không thể mở máy in: {:?}", e)))?;

    // Lần 1: lấy kích thước DEVMODE đầy đủ
    let size = DocumentPropertiesW(None, printer_handle, printer_name, None, None, 0);

    if size < 0 {
        ClosePrinter(printer_handle).ok();

        return Err(PrinterError::Message(
            "Không thể lấy kích thước DEVMODE.".into(),
        ));
    }

    let size = size as usize;

    // DEVMODE có thể chứa private data của driver,
    // nên phải giữ toàn bộ buffer.
    let mut buffer = vec![0u8; size];

    // Lần 2: lấy DEVMODE hiện tại của printer
    let result = DocumentPropertiesW(
        None,
        printer_handle,
        printer_name,
        Some(buffer.as_mut_ptr() as *mut DEVMODEW),
        None,
        DM_OUT_BUFFER,
    );

    ClosePrinter(printer_handle).ok();

    if result < 0 {
        return Err(PrinterError::Message(
            "Không thể lấy DEVMODE của máy in.".into(),
        ));
    }

    Ok(buffer)
}
