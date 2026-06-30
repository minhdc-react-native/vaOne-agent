use crate::utils::notification;
use printers::get_printers;
use std::process::Command;
#[tauri::command]
pub fn get_default_printer() -> Result<String, String> {
    let output = Command::new("lpstat")
        .arg("-d")
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("Failed to get default printer".into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // system default destination: Canon_G2010
    if let Some(name) = stdout.trim().strip_prefix("system default destination: ") {
        return Ok(name.to_string());
    }

    Err("No default printer found".into())
}

#[tauri::command]
pub fn get_printer_list() -> Result<Vec<String>, String> {
    let printers = get_printers();
    let names = printers.into_iter().map(|p| p.name).collect();
    Ok(names)
}

#[tauri::command]
pub fn print_pdf(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("lp")
            // .arg("-d")
            // .arg(&printer)
            .arg(&path)
            .status()
            .map_err(|e| {
                let msg = format!("Không thể gửi lệnh in: {}", e);
                let _ = notification::show("vaOne", &msg);
                e.to_string()
            })?;

        if !status.success() {
            let msg = format!("In PDF thất bại: {}", status);
            let _ = notification::show("vaOne", &msg);
            return Err(format!("Failed to print PDF. Exit status: {}", status));
        }
    }

    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("powershell")
            .args([
                "-Command",
                &format!("Start-Process -FilePath '{}' -Verb Print", path),
            ])
            .status()
            .map_err(|e| {
                let msg = format!("Không thể gửi lệnh in: {}", e);
                let _ = notification::show("vaOne", &msg);
                e.to_string()
            })?;

        if !status.success() {
            let msg = format!("In PDF thất bại: {}", status);
            let _ = notification::show("vaOne", &msg);
            return Err(format!("PowerShell exited with status: {}", status));
        }
    }

    Ok(())
}
