use crate::binder::DynamicContent;
use crate::fonts::PdfFonts;
use crate::pagination::page::PreparedReport;
use crate::utils::{get_formatter_context, Unit};
use crate::{layout::TextLayout, models::*, utils::load_fonts};
use printpdf::{PdfDocument, PdfSaveOptions};
use serde_json::{json, Value};

use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::ElementVecExt;
use crate::pagination::{
    layout_builder::LayoutBuilder,
    paginator::{PageItem, Paginator},
    PageRenderer,
};

pub fn render_bytes<F>(
    docs: Vec<PdfTemplate>,
    datas: Vec<serde_json::Value>,
    progress: &mut F,
) -> anyhow::Result<Vec<u8>>
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

    // Page number
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
        "current": 0
    }));

    // Lưu PDF trực tiếp vào bytes
    let mut warnings = Vec::new();

    let bytes = pdf.save(&PdfSaveOptions::default(), &mut warnings);

    Ok(bytes)
}

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
    // println!("prepared {:#?}", prepared);
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

    bind_total_pages(&mut report, fonts, start_page_number, total_pages_number);

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

fn bind_total_pages(
    report: &mut PreparedReport,
    fonts: &PdfFonts,
    start_page: usize,
    total_pages: usize,
) {
    let page_height = report.height;
    let formatter_context = report.ctx.clone();

    for (index, page) in report.pages.iter_mut().enumerate() {
        let current_page = start_page + index;
        let context = build_system_context(current_page, total_pages);
        for item in &mut page.items {
            let PageItem::Text { element, layout } = item else {
                continue;
            };

            let name = element.name.as_deref();

            let is_total_pages = name == Some("totalPages");
            let is_page_info = name.is_some_and(|name| name.starts_with("pageInfo"));

            if !is_total_pages && !is_page_info {
                continue;
            }

            // let mut watch = Vec::new();

            // if let Ok(dynamic) = serde_json::from_str::<DynamicContent>(&element.content) {
            //     watch = dynamic.watch;
            //     element.content = dynamic.fn_text;
            // }

            // context = TextLayout::build_context(&context, "", "value", &watch);

            let y = layout.y;
            let visible = layout.visible;
            let mut updated = TextLayout::layout(
                fonts,
                page_height,
                element,
                &context,
                formatter_context.clone(),
            );
            updated.y = y;
            updated.visible = visible;
            *layout = updated;
        }
    }
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

fn build_system_context(current_page: usize, total_pages: usize) -> Value {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Việt Nam = UTC+7
    let timestamp = now + 7 * 60 * 60;

    let seconds = timestamp % 60;
    let minutes = (timestamp / 60) % 60;
    let hours = (timestamp / 3600) % 24;

    // Tính ngày theo Unix timestamp
    let days = timestamp / 86400;

    // Thuật toán chuyển số ngày Unix thành ngày/tháng/năm
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if m <= 2 { 1 } else { 0 };

    json!({
        "value": format!("{:02}", total_pages),
        "currentPage":format!("{:02}", current_page),
        "totalPages":format!("{:02}", total_pages),
        "date": format!("{:02}/{:02}/{:04}", d, m, year),
        "time": format!("{:02}:{:02}:{:02}", hours, minutes, seconds),
        "dateTime": format!(
            "{:02}/{:02}/{:04} {:02}:{:02}:{:02}",
            d, m, year,
            hours, minutes, seconds
        ),

        "year": format!("{:04}", year),
        "month": format!("{:02}", m),
        "day": format!("{:02}", d),
    })
}
