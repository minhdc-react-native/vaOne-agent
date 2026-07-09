use crate::pdf::{layout::TextLayout, models::*, utils::load_fonts};
use printpdf::{PdfDocument, PdfSaveOptions};

use crate::pdf::pagination::{
    layout_builder::LayoutBuilder,
    paginator::{PageItem, Paginator},
    PageRenderer,
};
use crate::state::APP_HANDLE;
use tauri::Emitter;

fn emit_pdf_progress(progress: PdfProgress) {
    if let Some(app) = APP_HANDLE.get() {
        let _ = app.emit("pdf-progress", progress);
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

pub fn render_page(doc: PdfTemplate, data: serde_json::Value, output: &str) -> anyhow::Result<()> {
    let mut pdf = PdfDocument::new(&doc.name);

    let fonts = load_fonts(&mut pdf)?;

    // 1. Build layout
    let items = LayoutBuilder::build_items(&doc, &fonts, &data)?;

    // 2. Pagination
    let mut pages = Paginator::paginate(items, doc.width, doc.height)?;
    let total = pages.len();
    let context = serde_json::Value::Object(serde_json::Map::new());
    let style_text = Some(ElementStyle {
        color: Some("#808080".into()),
        font_size: Some(10.0),
        font_style: Some("italic".into()),
        text_align: Some("right".into()),
        ..Default::default()
    });
    for (index, page) in pages.iter_mut().enumerate() {
        let text = format!("Trang {}/{}", index + 1, total);
        let element = TextElement {
            name: None,
            x: doc.width - 100.0 - 35.0,
            y: doc.height - 35.0,
            width: 100.0,
            height: 24.0,
            content: text,
            field_name: None,
            style: style_text.clone(),
            auto_height: Some(false),
        };
        page.items.push(PageItem::Text {
            element: element.clone(),
            layout: TextLayout::layout(&fonts, doc.height, &element, &context),
        });
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
            emit_pdf_progress(PdfProgress {
                phase: PdfPhase::Rendering,
                current,
                total,
            });
        },
    )?;

    let mut warnings = Vec::new();

    let bytes = pdf.save(&PdfSaveOptions::default(), &mut warnings);

    std::fs::write(output, bytes)?;

    Ok(())
}
