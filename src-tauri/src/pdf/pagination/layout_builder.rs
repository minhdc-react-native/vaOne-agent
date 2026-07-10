use super::paginator::PageItem;
use crate::pdf::{
    fonts::PdfFonts,
    layout::TextLayout,
    models::{Element, PdfTemplate},
    table::table_layout::TableLayoutEngine,
    template::models::FormatterContext,
};
use serde_json::Value;
pub struct LayoutBuilder;

impl LayoutBuilder {
    pub fn build_items(
        doc: &PdfTemplate,
        fonts: &PdfFonts,
        data: &Value,
        ctx: FormatterContext,
    ) -> anyhow::Result<Vec<PageItem>> {
        let mut items = Vec::new();
        let mut current_offset = 0.0;
        for e in &doc.elements {
            match e {
                Element::Text(element) => {
                    let mut element = element.clone();
                    // element.translate_y(current_offset);
                    let context = if let Some(field) = element.field_name.as_deref() {
                        TextLayout::build_context(data, field, "value")
                    } else {
                        Value::Object(Default::default())
                    };

                    let layout =
                        TextLayout::layout(fonts, doc.height, &element, &context, ctx.clone());
                    current_offset += layout.height - element.height;

                    items.push(PageItem::Text { element, layout });
                }

                Element::Table(element) => {
                    let mut element = element.clone();
                    // element.translate_y(current_offset);
                    let layout =
                        TableLayoutEngine::build(fonts, doc.height, &element, data, ctx.clone());

                    current_offset += layout.height - element.height;
                    items.push(PageItem::Table { element, layout });
                }

                Element::Line(element) => {
                    let mut element = element.clone();
                    let layout = element.clone();
                    // element.translate_y(current_offset);
                    items.push(PageItem::Line { element, layout });
                }

                Element::Rect(element) => {
                    let mut element = element.clone();
                    let layout = element.clone();
                    // element.translate_y(current_offset);
                    items.push(PageItem::Rect { element, layout });
                }

                Element::Circle(element) => {
                    let mut element = element.clone();
                    let layout = element.clone();
                    // element.translate_y(current_offset);
                    items.push(PageItem::Circle { element, layout });
                }

                Element::Image(element) => {
                    let mut element = element.clone();
                    let layout = element.clone();
                    // element.translate_y(current_offset);
                    items.push(PageItem::Image { element, layout });
                }

                Element::Grid(element) => {
                    let mut element = element.clone();
                    let layout = element.clone();
                    // element.translate_y(current_offset);
                    items.push(PageItem::Grid { element, layout });
                }
            }
        }

        Ok(items)
    }
}
