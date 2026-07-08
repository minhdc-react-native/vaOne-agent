use super::page::PageLayout;
use crate::pdf::{
    fonts::PdfFonts,
    layout::TextLayout,
    models::*,
    table::{
        models::{TableElement, TableLayoutResult},
        table_layout::TableLayoutEngine,
    },
};
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum PageItem {
    Text {
        element: TextElement,
        layout: TextLayoutResult,
    },

    Table {
        element: TableElement,
        layout: TableLayoutResult,
    },

    Line {
        element: LRCElement,
    },

    Rect {
        element: LRCElement,
    },

    Circle {
        element: LRCElement,
    },

    Image {
        element: LRCElement,
    },

    Grid {
        element: GridElement,
    },
}
pub struct Paginator;

impl Paginator {
    pub fn build_items(
        doc: &PdfTemplate,
        fonts: &PdfFonts,
        data: &Value,
    ) -> anyhow::Result<Vec<PageItem>> {
        let mut items = Vec::new();
        let mut current_offset = 0.0;
        for e in &doc.elements {
            match e {
                Element::Text(element) => {
                    let mut element = element.clone();
                    element.y += current_offset;
                    let context = if let Some(field) = element.field_name.as_deref() {
                        TextLayout::build_context(data, field, "value")
                    } else {
                        serde_json::Value::Object(serde_json::Map::new())
                    };

                    let layout = TextLayout::layout(fonts, doc.height, &element, &context);
                    current_offset += layout.height - element.height;

                    items.push(PageItem::Text { element, layout });
                }
                Element::Table(element) => {
                    let mut element = element.clone();
                    element.y += current_offset;
                    let layout = TableLayoutEngine::build(&fonts, doc.height, &element, &data);
                    current_offset += layout.height - element.height;
                    items.push(PageItem::Table { element, layout });
                }

                Element::Line(element) => {
                    let mut element = element.clone();
                    element.y += current_offset;
                    let mut element = element.clone();
                    items.push(PageItem::Line { element });
                }

                Element::Rect(element) => {
                    let mut element = element.clone();
                    element.y += current_offset;
                    items.push(PageItem::Rect { element });
                }

                Element::Circle(element) => {
                    let mut element = element.clone();
                    element.y += current_offset;
                    items.push(PageItem::Circle { element });
                }

                Element::Image(element) => {
                    let mut element = element.clone();
                    element.y += current_offset;
                    items.push(PageItem::Image { element });
                }
                Element::Grid(element) => {
                    let mut element = element.clone();
                    element.y += current_offset;
                    items.push(PageItem::Grid { element });
                }
            }
        }
        Ok(items)
    }

    pub fn paginate(
        items: Vec<PageItem>,
        page_width: f32,
        page_height: f32,
    ) -> anyhow::Result<Vec<PageLayout>> {
    }
}
