use crate::pdf::{
    image::{render_background_image, render_image},
    layout::TextLayout,
    models::*,
    table::{table_layout::TableLayoutEngine, table_render::TableRenderer},
    text,
    utils::{draw_circle, draw_element_border, draw_line, load_fonts, Unit},
};
use printpdf::{Op, PdfDocument, PdfPage, PdfSaveOptions};
pub fn render(doc: PdfTemplate, data: serde_json::Value, output: &str) -> anyhow::Result<()> {
    let mut pdf = PdfDocument::new(&doc.name);
    let mut ops = Vec::<Op>::new();

    let _ = render_background_image(
        &mut pdf,
        &mut ops,
        doc.background_image,
        doc.width,
        doc.height,
    );

    let fonts: super::fonts::PdfFonts = load_fonts(&mut pdf)?;
    let mut current_offset = 0.0;
    for e in doc.elements {
        match e {
            Element::Text(mut element) => {
                element.y += current_offset;

                let context = if let Some(field) = element.field_name.as_deref() {
                    TextLayout::build_context(&data, field, "value")
                } else {
                    serde_json::Value::Object(serde_json::Map::new())
                };

                let layout = TextLayout::layout(&fonts, doc.height, &element, &context);

                let real = layout.height;
                let diff = real - element.height;

                if let Some(style) = &element.style {
                    draw_element_border(
                        &mut ops,
                        &fonts,
                        element.x,
                        layout.base_y + element.height - 4.0,
                        element.width,
                        element.height,
                        style,
                    );
                }

                text::draw_text(&mut ops, &fonts, &element, &layout);

                current_offset += diff;
            }

            Element::Table(mut element) => {
                element.y += current_offset;

                let layout = TableLayoutEngine::build(&fonts, doc.height, &element, &data);

                if let Some(style) = &element.style {
                    TableRenderer::draw(&mut ops, &layout, &fonts, doc.height, style);
                }

                let diff = layout.height - element.height;

                current_offset += diff;
            }

            Element::Line(mut element) => {
                element.y += current_offset;
                if let Some(style) = &element.style {
                    draw_line(
                        &mut ops,
                        &fonts,
                        element.x,
                        doc.height - element.y,
                        element.width,
                        element.height,
                        style,
                    );
                }
            }

            Element::Rect(mut element) => {
                element.y += current_offset;
                if let Some(name) = element.name.as_deref() {
                    if name == "rect_xh7w" || name == "****" {
                        println!(
                            "Rect>>{} {} {} {}",
                            name,
                            doc.height - element.y,
                            current_offset,
                            element.y
                        );
                    }
                }
                if let Some(style) = &element.style {
                    draw_element_border(
                        &mut ops,
                        &fonts,
                        element.x,
                        doc.height - element.y,
                        element.width,
                        element.height,
                        style,
                    );
                }
            }

            Element::Circle(mut element) => {
                element.y += current_offset;

                if let Some(style) = &element.style {
                    draw_circle(
                        &mut ops,
                        &fonts,
                        element.x,
                        doc.height - element.y,
                        element.width,
                        element.height,
                        style,
                    );
                }
            }

            Element::Image(mut element) => {
                element.y += current_offset;
                let _ = render_image(&mut pdf, &mut ops, &element, doc.height);
            }

            Element::Grid(mut element) => {}
        }
    }
    let page = PdfPage::new(Unit::px_to_mm(doc.width), Unit::px_to_mm(doc.height), ops);

    pdf.with_pages(vec![page]);

    let mut warnings = Vec::new();

    let bytes = pdf.save(&PdfSaveOptions::default(), &mut warnings);

    std::fs::write(output, bytes)?;

    Ok(())
}

use crate::pdf::pagination::{paginator::Paginator, PageRenderer};

pub fn render_page(doc: PdfTemplate, data: serde_json::Value, output: &str) -> anyhow::Result<()> {
    let mut pdf = PdfDocument::new(&doc.name);

    let fonts = load_fonts(&mut pdf)?;

    // 1. Build layout
    let items = Paginator::build_items(&doc, &fonts, &data)?;

    // 2. Pagination
    let pages = Paginator::paginate(items, doc.width, doc.height)?;

    // 3. Render
    let pdf_pages = PageRenderer::render(&mut pdf, &fonts, pages, doc.width, doc.height)?;

    pdf.with_pages(pdf_pages);

    let mut warnings = Vec::new();

    let bytes = pdf.save(&PdfSaveOptions::default(), &mut warnings);

    std::fs::write(output, bytes)?;

    Ok(())
}
