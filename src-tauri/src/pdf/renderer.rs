use crate::pdf::layout::TextLayout;
use crate::pdf::models::*;
use crate::pdf::table::table_layout::TableLayoutEngine;
use crate::pdf::table::table_render::TableRenderer;
use crate::pdf::text;
use crate::pdf::utils::{draw_element_border, load_fonts, Unit};
use printpdf::{Op, PdfDocument, PdfPage, PdfSaveOptions};
pub fn render(doc: PdfTemplate, data: serde_json::Value, output: &str) -> anyhow::Result<()> {
    let mut pdf = PdfDocument::new("Invoice");
    let mut ops = Vec::<Op>::new();

    let fonts = load_fonts(&mut pdf)?;
    let mut offset = 0.0;
    for e in doc.elements {
        match e {
            Element::Text(mut t) => {
                t.y += offset;
                let context = if let Some(field) = t.field_name.as_deref() {
                    TextLayout::build_context(&data, field, "value")
                } else {
                    serde_json::Value::Object(serde_json::Map::new())
                };

                let layout: TextLayoutResult = TextLayout::layout(&fonts, doc.height, &t, &context);
                let real = layout.height;
                let diff = (real - t.height).max(0.0);
                if let Some(style) = &t.style {
                    draw_element_border(
                        &mut ops,
                        &fonts,
                        t.x,
                        layout.base_y,
                        t.width,
                        t.height,
                        style,
                    );
                }
                text::draw_text(&mut ops, &fonts, &t, &layout);
                offset += diff;
            }
            Element::Table(mut t) => {
                t.y += offset;
                let layout = TableLayoutEngine::build(&fonts, doc.height, &t, &data);
                if let Some(style) = &t.style {
                    TableRenderer::draw(&mut ops, &layout, &fonts, doc.height, style);
                }
                let real = layout.height;
                let diff = (real - t.height).max(0.0);
                offset += diff;
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
