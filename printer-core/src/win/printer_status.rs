use crate::{PrinterError, PrinterStatus, Result};

use windows::{
    Win32::Graphics::Printing::{
        ClosePrinter, GetPrinterW, OpenPrinterW, PRINTER_ACCESS_USE, PRINTER_DEFAULTSW,
        PRINTER_INFO_2W, PRINTER_STATUS_BUSY, PRINTER_STATUS_DOOR_OPEN, PRINTER_STATUS_ERROR,
        PRINTER_STATUS_IO_ACTIVE, PRINTER_STATUS_MANUAL_FEED, PRINTER_STATUS_OFFLINE,
        PRINTER_STATUS_OUTPUT_BIN_FULL, PRINTER_STATUS_PAPER_JAM, PRINTER_STATUS_PAPER_OUT,
        PRINTER_STATUS_PAUSED, PRINTER_STATUS_PENDING_DELETION, PRINTER_STATUS_PRINTING,
        PRINTER_STATUS_PROCESSING, PRINTER_STATUS_TONER_LOW, PRINTER_STATUS_USER_INTERVENTION,
        PRINTER_STATUS_WAITING, PRINTER_STATUS_WARMING_UP,
    },
    core::{PCWSTR, PWSTR},
};

fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn get_printer_status(printer: &str) -> Result<PrinterStatus> {
    unsafe {
        let printer_name = to_wide(printer);

        let mut handle = Default::default();

        let defaults = PRINTER_DEFAULTSW {
            pDatatype: PWSTR(std::ptr::null_mut()),
            pDevMode: std::ptr::null_mut(),
            DesiredAccess: PRINTER_ACCESS_USE,
        };

        OpenPrinterW(PCWSTR(printer_name.as_ptr()), &mut handle, Some(&defaults))
            .map_err(|e| PrinterError::Message(e.to_string()))?;

        // Lấy kích thước buffer
        let mut needed = 0u32;

        let _ = GetPrinterW(handle, 2, None, &mut needed);

        if needed == 0 {
            ClosePrinter(handle);

            return Err(PrinterError::Message(
                "Không thể lấy thông tin máy in.".into(),
            ));
        }

        let mut buffer = vec![0u8; needed as usize];

        GetPrinterW(handle, 2, Some(buffer.as_mut_slice()), &mut needed).map_err(|e| {
            ClosePrinter(handle);
            PrinterError::Message(e.to_string())
        })?;

        let info = &*(buffer.as_ptr() as *const PRINTER_INFO_2W);

        let status = info.Status;

        println!("========== PRINTER_INFO_2 ==========");
        println!("Status      : 0x{:08X}", status);
        println!("Attributes  : 0x{:08X}", info.Attributes);
        println!("Jobs        : {}", info.cJobs);
        println!("AveragePPM  : {}", info.AveragePPM);
        println!("Priority    : {}", info.Priority);
        println!("DefaultPrio : {}", info.DefaultPriority);
        println!("====================================");

        let mut reasons = Vec::<String>::new();
        let mut messages = Vec::<String>::new();

        macro_rules! add_reason {
            ($flag:expr, $key:expr, $message:expr) => {
                if status & $flag != 0 {
                    reasons.push($key.to_string());
                }
            };
        }

        // -----------------------------
        // Tất cả trạng thái
        // -----------------------------

        add_reason!(PRINTER_STATUS_OFFLINE, "offline", "Máy in đang ngoại tuyến");

        add_reason!(PRINTER_STATUS_PAPER_OUT, "media-empty", "Máy in hết giấy");

        add_reason!(PRINTER_STATUS_PAPER_JAM, "media-jam", "Máy in bị kẹt giấy");

        add_reason!(PRINTER_STATUS_DOOR_OPEN, "door-open", "Nắp máy in đang mở");

        add_reason!(PRINTER_STATUS_PAUSED, "paused", "Máy in đang tạm dừng");

        add_reason!(PRINTER_STATUS_TONER_LOW, "toner-low", "Mực in sắp hết");

        add_reason!(PRINTER_STATUS_ERROR, "error", "Máy in đang gặp lỗi");

        add_reason!(PRINTER_STATUS_BUSY, "busy", "Máy in đang bận");

        add_reason!(PRINTER_STATUS_PRINTING, "printing", "Máy in đang in");

        add_reason!(PRINTER_STATUS_PROCESSING, "processing", "Máy in đang xử lý");

        add_reason!(
            PRINTER_STATUS_USER_INTERVENTION,
            "user-intervention",
            "Cần người dùng can thiệp"
        );

        add_reason!(
            PRINTER_STATUS_OUTPUT_BIN_FULL,
            "output-full",
            "Khay giấy đầu ra đã đầy"
        );

        add_reason!(
            PRINTER_STATUS_MANUAL_FEED,
            "manual-feed",
            "Máy in đang chờ nạp giấy thủ công"
        );

        add_reason!(
            PRINTER_STATUS_IO_ACTIVE,
            "io-active",
            "Máy in đang truyền dữ liệu"
        );

        add_reason!(PRINTER_STATUS_WAITING, "waiting", "Máy in đang chờ");

        add_reason!(
            PRINTER_STATUS_WARMING_UP,
            "warming-up",
            "Máy in đang khởi động"
        );

        add_reason!(
            PRINTER_STATUS_PENDING_DELETION,
            "pending-delete",
            "Máy in đang chờ xóa"
        );

        // -----------------------------
        // Những reason thực sự là lỗi
        // -----------------------------

        let messages = get_error_messages(status);

        // -----------------------------
        // Xác định has_error
        // -----------------------------

        let has_error = !messages.is_empty();

        // Windows không có flag tương đương trực tiếp
        // với printer-is-accepting-jobs của CUPS.
        //
        // Tạm coi printer không nhận job khi:
        // - paused
        // - pending deletion
        let accepting =
            status & PRINTER_STATUS_PAUSED == 0 && status & PRINTER_STATUS_PENDING_DELETION == 0;

        ClosePrinter(handle);

        Ok(PrinterStatus {
            state: status,
            reasons,
            messages,
            accepting,
            has_error,
        })
    }
}

fn get_error_messages(status: u32) -> Vec<String> {
    let mut messages = Vec::new();

    if status & PRINTER_STATUS_OFFLINE != 0 {
        messages.push("Máy in đang ngoại tuyến".into());
    }

    if status & PRINTER_STATUS_PAPER_OUT != 0 {
        messages.push("Máy in hết giấy".into());
    }

    if status & PRINTER_STATUS_PAPER_JAM != 0 {
        messages.push("Máy in bị kẹt giấy".into());
    }

    if status & PRINTER_STATUS_DOOR_OPEN != 0 {
        messages.push("Nắp máy in đang mở".into());
    }

    if status & PRINTER_STATUS_PAUSED != 0 {
        messages.push("Máy in đang tạm dừng".into());
    }

    if status & PRINTER_STATUS_ERROR != 0 {
        messages.push("Máy in đang gặp lỗi".into());
    }

    if status & PRINTER_STATUS_USER_INTERVENTION != 0 {
        messages.push("Cần người dùng can thiệp".into());
    }

    if status & PRINTER_STATUS_OUTPUT_BIN_FULL != 0 {
        messages.push("Khay giấy đầu ra đã đầy".into());
    }

    if status & PRINTER_STATUS_MANUAL_FEED != 0 {
        messages.push("Máy in đang chờ nạp giấy thủ công".into());
    }

    messages
}
