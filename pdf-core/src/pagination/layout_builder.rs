use crate::binder::resolve_value;
use crate::pagination::paginator::PageItem;
use crate::template::evaluator::Evaluator;
use crate::{
    binder::DynamicContent,
    fonts::PdfFonts,
    layout::TextLayout,
    models::{Element, PdfTemplate},
    table::table_layout::TableLayoutEngine,
    template::models::FormatterContext,
};
use serde_json::{json, Value};
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
            let visible_if = e.visible_if();
            let visible = Evaluator::evaluate_visible_if(visible_if, data);

            match e {
                Element::Text(element) => {
                    let mut element = element.clone();
                    if element.content.trim().is_empty() {
                        element.content = "{value}".to_string();
                    }

                    let context = if let Some(name) = element.name.as_deref() {
                        if name == "totalPages" {
                            json!({ "value": "00" })
                        } else if name.starts_with("pageInfo") {
                            json!({
                                "totalPage": 99,
                                "date": "01/01/0000",
                                "time": "01:00",
                                "dateTime": "01/01/0000 01:00"
                            })
                        } else {
                            let mut watch = Vec::new();

                            match serde_json::from_str::<DynamicContent>(&element.content) {
                                Ok(dynamic) => {
                                    watch = dynamic.watch;
                                    element.content = dynamic.fn_text;
                                }
                                Err(_) => {}
                            }

                            if let Some(field) = element
                                .field_name
                                .as_deref()
                                .or_else(|| watch.first().map(String::as_str))
                            {
                                TextLayout::build_context(data, field, "value", &watch)
                            } else {
                                Value::Object(Default::default())
                            }
                        }
                    } else {
                        Value::Object(Default::default())
                    };

                    let mut layout: crate::models::TextLayoutResult =
                        TextLayout::layout(fonts, doc.height, &element, &context, ctx.clone());
                    layout.visible = Some(visible);

                    current_offset += layout.height - element.height;

                    items.push(PageItem::Text { element, layout });
                }

                Element::Table(element) => {
                    let element = element.clone();
                    // element.translate_y(current_offset);
                    let mut layout =
                        TableLayoutEngine::build(fonts, doc.height, &element, data, ctx.clone());
                    layout.visible = Some(visible);
                    current_offset += layout.height - element.height;
                    items.push(PageItem::Table { element, layout });
                }

                Element::Line(element) => {
                    let element = element.clone();
                    let mut layout = element.clone();
                    layout.visible = Some(visible);
                    // element.translate_y(current_offset);
                    items.push(PageItem::Line { element, layout });
                }

                Element::Rect(element) => {
                    let element = element.clone();
                    let mut layout = element.clone();
                    layout.visible = Some(visible);
                    // element.translate_y(current_offset);
                    items.push(PageItem::Rect { element, layout });
                }

                Element::Circle(element) => {
                    let element = element.clone();
                    let mut layout = element.clone();
                    layout.visible = Some(visible);
                    // element.translate_y(current_offset);
                    items.push(PageItem::Circle { element, layout });
                }

                Element::Image(element) => {
                    let element = element.clone();
                    let mut layout = element.clone();

                    if let Some(field_name) = &element.field_name {
                        if !field_name.trim().is_empty() {
                            layout.content = resolve_value(data, field_name);
                        }
                    }
                    layout.visible = Some(visible);
                    items.push(PageItem::Image { element, layout });
                }

                Element::Grid(element) => {
                    let element = element.clone();
                    let mut layout = element.clone();
                    layout.visible = Some(visible);
                    // element.translate_y(current_offset);
                    items.push(PageItem::Grid { element, layout });
                }
            }
        }

        Ok(items)
    }
}
