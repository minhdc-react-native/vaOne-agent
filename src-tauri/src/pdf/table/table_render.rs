use printpdf::Op;

use crate::pdf::fonts::PdfFonts;

use crate::pdf::border::Border;
use crate::pdf::table::models::{TableCellLayout, TableLayoutResult};

pub struct TableRenderer;

impl TableRenderer {
    pub fn draw(ops: &mut Vec<Op>, layout: &TableLayoutResult, fonts: &PdfFonts) {
        for row in &layout.rows {
            for cell in &row.cells {
                Self::draw_cell(ops, cell, fonts);
            }
        }
    }

    fn draw_cell(ops: &mut Vec<Op>, cell: &TableCellLayout, fonts: &PdfFonts) {
        //--------------------------------------------------
        // Background
        //--------------------------------------------------

        Self::draw_background(ops, cell);

        //--------------------------------------------------
        // Border
        //--------------------------------------------------

        Border::draw(
            ops,
            fonts,
            cell.x,
            cell.y,
            cell.width,
            cell.height,
            cell.style.border_radius.unwrap_or(0.0),
            None,
            None,
            None,
            None,
        );

        //--------------------------------------------------
        // Text
        //--------------------------------------------------

        Text::draw(
            ops,
            cell.x,
            cell.y,
            cell.width,
            cell.height,
            &cell.runs,
            &cell.style,
            fonts,
        );
    }

    fn draw_background(ops: &mut Vec<Op>, cell: &TableCellLayout) {
        let Some(color) = &cell.style.background_color else {
            return;
        };

        if color.eq_ignore_ascii_case("transparent") {
            return;
        }

        // TODO:
        // Sau này gọi Rect::fill(...)
        //
        // Rect::fill(
        //      ops,
        //      cell.x,
        //      cell.y,
        //      cell.width,
        //      cell.height,
        //      color,
        // );
    }
}
