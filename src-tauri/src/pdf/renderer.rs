use crate::pdf::utils::get_formatter_context;
use crate::pdf::{layout::TextLayout, models::*, utils::load_fonts};
use printpdf::{PdfDocument, PdfSaveOptions};
use serde_json::json;

use crate::pdf::models::ElementVecExt;
use crate::pdf::pagination::{
    layout_builder::LayoutBuilder,
    paginator::{PageItem, Paginator},
    PageRenderer,
};

use crate::state::APP_HANDLE;
use tauri::Emitter;

fn emit_pdf_progress(progress: serde_json::Value) {
    if let Some(app) = APP_HANDLE.get() {
        let _ = app.emit("pdf-progress", progress);
        // std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

pub fn render_page(
    mut doc: PdfTemplate,
    data: serde_json::Value,
    output: &str,
) -> anyhow::Result<()> {
    let (page_number, elements) = std::mem::take(&mut doc.elements).extract_page_number();
    doc.elements = elements;
    doc.elements.sort_by_y();

    let ctx = get_formatter_context(&data);

    let mut pdf = PdfDocument::new(&doc.name);

    let fonts = load_fonts(&mut pdf)?;

    // 1. Build layout
    emit_pdf_progress(json!({
        "message": "Xây dựng giao diện...",
    }));
    let items = LayoutBuilder::build_items(&doc, &fonts, &data, ctx.clone())?;

    emit_pdf_progress(json!({
        "message": "Tính toán phân trang...",
    }));
    // 2. Pagination
    let mut pages = Paginator::paginate(items, doc.width, doc.height)?;
    if let Some(page_number) = &page_number {
        let total = pages.len();
        let element = page_number.as_text();
        if let Some(element) = element {
            for (index, page) in pages.iter_mut().enumerate() {
                let context = json!({
                    "page":index+1,
                    "total":total
                });
                page.items.push(PageItem::Text {
                    element: element.clone(),
                    layout: TextLayout::layout(&fonts, doc.height, &element, &context, ctx.clone()),
                });
            }
        }
    }
    // 3. Render
    // let pdf_pages = PageRenderer::render(&mut pdf, &fonts, pages, doc.width, doc.height)?;

    // pdf.with_pages(pdf_pages);

    PageRenderer::render(
        &mut pdf,
        &fonts,
        pages,
        doc.width,
        doc.height,
        |current, total| {
            emit_pdf_progress(json!({
                "message": "",
                "current": current,
                "total": total,
            }));
        },
        ctx,
    )?;

    let mut warnings = Vec::new();

    let bytes = pdf.save(&PdfSaveOptions::default(), &mut warnings);

    std::fs::write(output, bytes)?;

    Ok(())
}
