use printpdf::Op;

use crate::{
    fonts::PdfFonts,
    layout::TextLayout,
    models::{ElementStyle, TextElement, TextLayoutResult},
    table::{
        models::{TableCellLayout, TableLayoutResult, TableRowLayout},
        rect::Rect,
        table_border::TableBorder,
    },
    template::models::FormatterContext,
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
        ctx: FormatterContext,
    ) {
        TableBorder::draw(ops, fonts, layout, style, page_height);

        let border_width = style.border_width.unwrap_or(0.0);
        // header
        for row in &layout.headers {
            Self::draw_row(ops, row, fonts, border_width, page_height, ctx.clone());
        }
        //body
        for row in &layout.rows {
            Self::draw_row(ops, row, fonts, border_width, page_height, ctx.clone());
        }
    }
    fn draw_row(
        ops: &mut Vec<Op>,
        row: &TableRowLayout,
        fonts: &PdfFonts,
        border_width: f32,
        page_height: f32,
        ctx: FormatterContext,
    ) {
        for cell in &row.cells {
            Self::draw_cell(ops, cell, fonts, border_width, page_height, ctx.clone());
        }
    }
    fn draw_cell(
        ops: &mut Vec<Op>,
        cell: &TableCellLayout,
        fonts: &PdfFonts,
        border_width: f32,
        page_height: f32,
        ctx: FormatterContext,
    ) {
        //--------------------------------------------------
        // Background
        //--------------------------------------------------

        Self::draw_background(ops, cell, fonts, border_width, page_height);

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
            visible_if: None,
        };

        let context = serde_json::json!({});
        let layout: TextLayoutResult =
            TextLayout::layout(&fonts, page_height, &text, &context, ctx);
        text::draw_text(ops, &fonts, &text, &layout, page_height);
    }

    fn draw_background(
        ops: &mut Vec<Op>,
        cell: &TableCellLayout,
        fonts: &PdfFonts,
        border_width: f32,
        page_height: f32,
    ) {
        const DEFAULT_HEADER_BACKGROUND: &str = "#fafafa";

        let color = if cell.is_row {
            // Row bình thường: chỉ tô nếu có màu và không phải transparent
            match cell.style.background_color.as_deref() {
                Some(c) if !c.eq_ignore_ascii_case("transparent") => Some(c),
                _ => None,
            }
        } else {
            // Header: mặc định #fafafa nếu không có màu
            Some(match cell.style.background_color.as_deref() {
                Some(c) if !c.eq_ignore_ascii_case("transparent") => c,
                _ => DEFAULT_HEADER_BACKGROUND,
            })
        };
        if let Some(color) = color {
            Rect::fill(
                ops,
                fonts,
                cell.x + border_width / 2.0,
                page_height - cell.y - cell.height + border_width / 2.0,
                cell.width - border_width,
                cell.height - border_width,
                color,
            );
        }
    }
}
