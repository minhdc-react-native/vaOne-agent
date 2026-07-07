use crate::pdf::utils::resolve_value;
use crate::pdf::{
    fonts::PdfFonts,
    layout::TextLayout,
    models::{ElementStyle, TextElement},
    table::{
        models::{TableCellLayout, TableColumn, TableElement, TableRowLayout},
        table_layout::TableLayoutEngine,
    },
};
use serde_json::Value;
pub struct TableRow;
const DEFAULT_ROW_HEIGHT: f32 = 0.0;
impl TableRow {
    pub fn build_rows(
        fonts: &PdfFonts,
        page_height: f32,
        table: &TableElement,
        widths: &[f32],
        positions: &[f32],
        start_y: f32,
        data: &[Value],
    ) -> Vec<TableRowLayout> {
        let mut rows = Vec::new();

        let row_height = DEFAULT_ROW_HEIGHT;

        let mut current_y = start_y;
        for item in data {
            let row = Self::build_row(
                fonts,
                page_height,
                table,
                item,
                widths,
                positions,
                current_y,
                row_height,
            );
            current_y += row.height;
            rows.push(row);
        }

        rows
    }

    fn build_row(
        fonts: &PdfFonts,
        page_height: f32,
        table: &TableElement,
        data: &Value,
        widths: &[f32],
        positions: &[f32],
        y: f32,
        row_height: f32,
    ) -> TableRowLayout {
        let mut row = TableRowLayout {
            y,

            height: row_height,

            cells: Vec::new(),
        };

        for (index, column) in table.columns.iter().enumerate() {
            row.cells.push(Self::build_cell(
                column,
                data,
                positions[index],
                y,
                widths[index],
                &table.style,
            ));
        }
        let row_height = Self::measure_row_height(
            fonts,
            page_height,
            &table.columns,
            data,
            widths,
            &table.style,
        );
        for cell in &mut row.cells {
            cell.height = row_height;
        }
        row.height = row_height;
        row
    }

    fn build_cell(
        column: &TableColumn,
        data: &Value,
        x: f32,
        y: f32,
        width: f32,
        table_style: &Option<ElementStyle>,
    ) -> TableCellLayout {
        let style = TableLayoutEngine::merge_style(table_style, &column.body_style);

        let value = resolve_value(data, &column.field_name)
            .map(|v| {
                if v.is_string() {
                    v.as_str().unwrap().to_string()
                } else {
                    v.to_string()
                }
            })
            .unwrap_or_default();

        TableCellLayout {
            x,
            y,
            width,
            height: 0.0,
            row_span: 1,
            col_span: 1,
            content: value,
            style,
            is_row: true,
        }
    }

    fn measure_row_height(
        fonts: &PdfFonts,
        page_height: f32,
        columns: &[TableColumn],
        data: &Value,
        widths: &[f32],
        table_style: &Option<ElementStyle>,
    ) -> f32 {
        let mut max_height = DEFAULT_ROW_HEIGHT;

        for (index, column) in columns.iter().enumerate() {
            let style = TableLayoutEngine::merge_style(table_style, &column.body_style);

            let value = resolve_value(data, &column.field_name)
                .map(|v| {
                    if let Some(s) = v.as_str() {
                        s.to_string()
                    } else {
                        v.to_string()
                    }
                })
                .unwrap_or_default();

            let text = TextElement {
                name: None,
                x: 0.0,
                y: 0.0,
                width: widths[index],
                height: 0.0,
                content: value,
                field_name: None,
                style: Some(style),
                auto_height: Some(true),
            };

            let layout = TextLayout::layout(fonts, page_height, &text, &serde_json::json!({}));

            // padding trên + dưới
            let cell_height = layout.height + 6.0;

            max_height = max_height.max(cell_height);
        }

        max_height
    }
}
