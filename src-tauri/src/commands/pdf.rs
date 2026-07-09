use crate::pdf::binder;
use crate::pdf::models::PdfTemplate;
use crate::pdf::renderer;

#[tauri::command]
pub async fn render_pdf(report: PdfTemplate, data: serde_json::Value) -> Result<String, String> {
    let report = binder::bind_template(report, &data);

    let path = dirs::download_dir().unwrap().join("invoice.pdf");
    let output = path.to_string_lossy().to_string();

    let output_clone = output.clone();

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        renderer::render_page(report, data, &output_clone).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(output)
}
