use crate::pdf::layout::TextLayout;
use crate::pdf::models::*;
use crate::pdf::table::table_layout::TableLayoutEngine;
use crate::pdf::table::table_render::TableRenderer;
use crate::pdf::text;
use crate::pdf::utils::{draw_circle, draw_element_border, draw_line, load_fonts, Unit};
use printpdf::{Op, PdfDocument, PdfPage, PdfSaveOptions};
const MINUS_HEIGHT_SHAP: f32 = 10.0;
pub fn render(doc: PdfTemplate, data: serde_json::Value, output: &str) -> anyhow::Result<()> {
    let mut pdf = PdfDocument::new("Invoice");
    let mut ops = Vec::<Op>::new();

    let fonts = load_fonts(&mut pdf)?;
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

                if let Some(name) = element.name.as_deref() {
                    if name == "text_son3" || name == "****" {
                        println!(
                            "text>>{} {} {} {}",
                            name, layout.base_y, current_offset, element.y
                        );
                    }
                }

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
        }
    }
    let page = PdfPage::new(Unit::px_to_mm(doc.width), Unit::px_to_mm(doc.height), ops);

    pdf.with_pages(vec![page]);

    let mut warnings = Vec::new();

    let bytes = pdf.save(&PdfSaveOptions::default(), &mut warnings);

    std::fs::write(output, bytes)?;

    Ok(())
}
