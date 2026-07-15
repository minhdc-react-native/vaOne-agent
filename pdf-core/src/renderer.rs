use crate::fonts::PdfFonts;
use crate::pagination::page::PreparedReport;
use crate::utils::{get_formatter_context, Unit};
use crate::{layout::TextLayout, models::*, utils::load_fonts};
use printpdf::{PdfDocument, PdfSaveOptions};
use serde_json::{json, Value};

use crate::models::ElementVecExt;
use crate::pagination::{
    layout_builder::LayoutBuilder,
    paginator::{PageItem, Paginator},
    PageRenderer,
};

// use crate::state::APP_HANDLE;
// // use tauri::Emitter;

// fn emit_pdf_progress(progress: serde_json::Value) {
//     if let Some(app) = APP_HANDLE.get() {
//         let _ = app.emit("pdf-progress", progress);
//         std::thread::sleep(std::time::Duration::from_millis(50));
//     }
// }

pub fn render_page<F>(
    docs: Vec<PdfTemplate>,
    datas: Vec<serde_json::Value>,
    output: &str,
    progress: &mut F,
) -> anyhow::Result<()>
where
    F: FnMut(serde_json::Value),
{
    anyhow::ensure!(!docs.is_empty(), "docs is empty");

    let mut pdf = PdfDocument::new("Report");
    let fonts = load_fonts(&mut pdf)?;

    let mut prepared = Vec::new();

    for (i, data) in datas.into_iter().enumerate() {
        let doc = docs.get(i).cloned().unwrap_or_else(|| docs[0].clone());

        prepared.push(prepare_report(doc, data, &fonts)?);
    }

    // Tính tổng số trang
    let total_pages: usize = prepared.iter().map(|r| r.pages.len()).sum();

    // Render
    let mut start_page = 1;
    let total = prepared.len();
    // page number
    let mut start_page_number = 1;
    let mut total_pages_number = total_pages;

    for (index, report) in prepared.into_iter().enumerate() {
        let continuous_page_numbering = report.ctx.continuous_page_numbering;
        if !continuous_page_numbering {
            total_pages_number = report.pages.len();
        }
        progress(json!({
            "currentReport": index + 1,
            "totalReport": total,
        }));
        start_page = render_single(
            &mut pdf,
            &fonts,
            report,
            start_page,
            total_pages,
            start_page_number,
            total_pages_number,
            progress,
        )?;
        start_page_number = start_page;
        if !continuous_page_numbering {
            start_page_number = 1;
        }
    }
    progress(json!({
        "message": "Đang lưu file pdf...",
        "current":0
    }));
    let mut warnings = Vec::new();
    let bytes = pdf.save(&PdfSaveOptions::default(), &mut warnings);
    std::fs::write(output, bytes)?;

    Ok(())
}

fn render_single<F>(
    pdf: &mut PdfDocument,
    fonts: &PdfFonts,
    mut report: PreparedReport,
    start_page: usize,
    total_pages: usize,
    start_page_number: usize,
    total_pages_number: usize,
    progress: &mut F,
) -> anyhow::Result<usize>
where
    F: FnMut(serde_json::Value),
{
    let page_count = report.pages.len();

    if let Some(element) = &report.page_number {
        for (index, page) in report.pages.iter_mut().enumerate() {
            let context = json!({
                "page": start_page_number + index,
                "total": total_pages_number,
            });
            page.items.push(PageItem::Text {
                element: element.clone(),
                layout: TextLayout::layout(
                    fonts,
                    report.height,
                    element,
                    &context,
                    report.ctx.clone(),
                ),
            });
        }
    }

    PageRenderer::render(
        pdf,
        fonts,
        report.pages,
        report.width,
        report.height,
        report.ctx,
        report.background_image,
        start_page,
        total_pages,
        progress,
    )?;

    Ok(start_page + page_count)
}

fn is_continuous_page(width_px: f32, height_px: f32) -> bool {
    let width = Unit::px_to_mm(width_px).0;
    let height = Unit::px_to_mm(height_px).0;

    const EPSILON: f32 = 2.0;

    const PAPER_SIZES: &[(f32, f32)] = &[
        (420.0, 594.0), // A2
        (297.0, 420.0), // A3
        (210.0, 297.0), // A4
        (148.0, 210.0), // A5
        (105.0, 148.0), // A6
        (216.0, 279.0), // Letter
        (216.0, 356.0), // Legal
    ];

    !PAPER_SIZES.iter().any(|&(w, h)| {
        ((width - w).abs() <= EPSILON && (height - h).abs() <= EPSILON)
            || ((width - h).abs() <= EPSILON && (height - w).abs() <= EPSILON)
    })
}

fn prepare_report(
    mut doc: PdfTemplate,
    data: Value,
    fonts: &PdfFonts,
) -> anyhow::Result<PreparedReport> {
    let (page_number, elements) = std::mem::take(&mut doc.elements).extract_page_number();

    doc.elements = elements;
    doc.elements.sort_by_y();

    let ctx = get_formatter_context(&data);

    let items = LayoutBuilder::build_items(&doc, fonts, &data, ctx.clone())?;

    let continuous = is_continuous_page(doc.width, doc.height);

    let (pages, height) = Paginator::paginate(items, doc.width, doc.height, continuous)?;

    Ok(PreparedReport {
        pages,
        ctx: ctx,
        page_number: if continuous {
            None
        } else {
            page_number.and_then(|p| p.as_text().cloned())
        },
        width: doc.width,
        height,
        background_image: doc.background_image,
    })
}
