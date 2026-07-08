use printpdf::Op;

use crate::pdf::{
    fonts::PdfFonts,
    layout::TextLayout,
    models::{ElementStyle, TextElement, TextLayoutResult},
    table::models::{TableCellLayout, TableLayoutResult, TableRowLayout},
    table::rect::Rect,
    table::table_border::TableBorder,
    text,
};
pub struct TableRenderer;

impl TableRenderer {
    pub fn draw(
        ops: &mut Vec<Op>,
        layout: &TableLayoutResult,
        fonts: &PdfFonts,
        page_height: f32,
        style: &ElementStyle,
    ) {
        TableBorder::draw(ops, fonts, layout, style, page_height);
        // header
        for row in &layout.headers {
            Self::draw_row(ops, row, fonts, page_height);
        }
        //body
        for row in &layout.rows {
            Self::draw_row(ops, row, fonts, page_height);
        }
    }
    fn draw_row(ops: &mut Vec<Op>, row: &TableRowLayout, fonts: &PdfFonts, page_height: f32) {
        for cell in &row.cells {
            Self::draw_cell(ops, cell, fonts, page_height);
        }
    }
    fn draw_cell(ops: &mut Vec<Op>, cell: &TableCellLayout, fonts: &PdfFonts, page_height: f32) {
        //--------------------------------------------------
        // Background
        //--------------------------------------------------

        Self::draw_background(ops, cell, fonts, page_height);

        //--------------------------------------------------
        // Text
        //--------------------------------------------------
        let text = TextElement {
            name: None,
            x: cell.x + 2.0,
            y: cell.y + if cell.is_row { 2.0 } else { 0.0 },
            width: cell.width - 4.0,
            height: cell.height,
            content: cell.content.clone(),
            field_name: None,
            style: Some(cell.style.clone()),
            auto_height: Some(true),
        };

        let context = serde_json::json!({});
        let layout: TextLayoutResult = TextLayout::layout(&fonts, page_height, &text, &context);
        text::draw_text(ops, &fonts, &text, &layout);
    }

    fn draw_background(
        ops: &mut Vec<Op>,
        cell: &TableCellLayout,
        fonts: &PdfFonts,
        page_height: f32,
    ) {
        if cell.is_row {
            return;
        }

        const DEFAULT_HEADER_BACKGROUND: &str = "#fafafa";

        let color = match cell.style.background_color.as_deref() {
            Some(c) if !c.eq_ignore_ascii_case("transparent") => c,
            _ => DEFAULT_HEADER_BACKGROUND,
        };

        Rect::fill(
            ops,
            fonts,
            cell.x + 0.5,
            page_height - cell.y - cell.height + 0.5,
            cell.width - 1.0,
            cell.height - 1.0,
            color,
        );
    }
}
