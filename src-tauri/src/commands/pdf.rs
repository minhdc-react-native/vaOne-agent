use crate::pdf::models::PdfTemplate;
use crate::pdf::renderer;
#[tauri::command]
pub fn render_pdf(document: PdfTemplate) -> Result<String, String> {
    let path = dirs::download_dir().unwrap().join("invoice.pdf");

    renderer::render(document, path.to_str().unwrap()).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}
