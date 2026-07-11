use crate::pdf::fonts::PdfFonts;
use crate::pdf::pagination::page::PreparedReport;
use crate::pdf::utils::get_formatter_context;
use crate::pdf::{layout::TextLayout, models::*, utils::load_fonts};
use printpdf::{PdfDocument, PdfSaveOptions};
use serde_json::{json, Value};

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
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

pub fn render_page(
    docs: Vec<PdfTemplate>,
    datas: Vec<serde_json::Value>,
    output: &str,
) -> anyhow::Result<()> {
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
        emit_pdf_progress(json!({
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
        )?;
        start_page_number = start_page;
        if !continuous_page_numbering {
            start_page_number = 1;
        }
    }
    emit_pdf_progress(json!({
        "message": "Đang lưu file pdf...",
        "current":0
    }));
    let mut warnings = Vec::new();
    let bytes = pdf.save(&PdfSaveOptions::default(), &mut warnings);
    std::fs::write(output, bytes)?;

    Ok(())
}

fn render_single(
    pdf: &mut PdfDocument,
    fonts: &PdfFonts,
    mut report: PreparedReport,
    start_page: usize,
    total_pages: usize,
    start_page_number: usize,
    total_pages_number: usize,
) -> anyhow::Result<usize> {
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
        |current, total| {
            emit_pdf_progress(json!({
                "message": "",
                "current": current,
                "total": total,
            }));
        },
        report.ctx,
        report.background_image,
        start_page,
        total_pages,
    )?;

    Ok(start_page + page_count)
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

    let pages = Paginator::paginate(items, doc.width, doc.height)?;

    Ok(PreparedReport {
        pages,
        ctx: ctx,
        page_number: page_number.and_then(|p| p.as_text().cloned()),
        width: doc.width,
        height: doc.height,
        background_image: doc.background_image,
    })
}
