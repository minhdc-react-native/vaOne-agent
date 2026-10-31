use pdf_core::models::PdfTemplate;
use pdf_core::renderer;

use crate::state::APP_HANDLE;
use tauri::Emitter;

#[tauri::command]
pub async fn render_pdf(
    reports: Vec<PdfTemplate>,
    datas: Vec<serde_json::Value>,
) -> Result<String, String> {
    let path = dirs::download_dir().unwrap().join("invoice.pdf");
    let output = path.to_string_lossy().to_string();

    let output_clone = output.clone();

    let mut progress = |p: serde_json::Value| {
        if let Some(app) = APP_HANDLE.get() {
            let _ = app.emit("pdf-progress", p);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    };

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        renderer::render_page(reports, datas, &output_clone, &mut progress)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(output)
}
