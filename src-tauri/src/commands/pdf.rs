use crate::pdf::binder;
use crate::pdf::models::PdfTemplate;
use crate::pdf::renderer;

#[tauri::command]
pub fn render_pdf(report: PdfTemplate, data: serde_json::Value) -> Result<String, String> {
    let report = binder::bind_template(report, &data);

    let path = dirs::download_dir().unwrap().join("invoice.pdf");

    renderer::render(report, path.to_str().unwrap()).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}
