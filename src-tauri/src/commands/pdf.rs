use crate::pdf::binder;
use crate::pdf::models::PdfTemplate;
use crate::pdf::renderer;

#[tauri::command]
pub async fn render_pdf(
    reports: Vec<PdfTemplate>,
    datas: Vec<serde_json::Value>,
) -> Result<String, String> {
    let reports = reports
        .into_iter()
        .enumerate()
        .map(|(i, report)| {
            let data = datas.get(i).unwrap_or(&datas[0]);
            binder::bind_template(report, data)
        })
        .collect::<Vec<_>>();

    let path = dirs::download_dir().unwrap().join("invoice.pdf");
    let output = path.to_string_lossy().to_string();

    let output_clone = output.clone();

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        renderer::render_page(reports, datas, &output_clone).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())??;

    Ok(output)
}
