use crate::table::style_class_name::class_name_to_style;
use crate::template::formatter::FORMATTERS;
use crate::template::models::FormatterContext;
use crate::utils::resolve_value;
use crate::{
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
        ctx: FormatterContext,
    ) -> Vec<TableRowLayout> {
        let mut rows = Vec::new();

        let mut current_y = start_y;
        for item in data {
            // A fixed-row template describes the physical lines of ONE data item.
            // Keep them in one logical row so pagination cannot separate the name
            // from its quantity/price line.
            let mut row = TableRowLayout {
                y: current_y,
                height: 0.0,
                cells: Vec::new(),
            };
            if let Some(fix) = table.fix_row.as_ref().filter(|fix| !fix.data.is_empty()) {
                for row_config in &fix.data {
                    let line = Self::build_row(
                        fonts,
                        page_height,
                        table,
                        &row_config.columns,
                        item,
                        widths,
                        positions,
                        current_y + row.height,
                        ctx.clone(),
                    );
                    row.height += line.height;
                    row.cells.extend(line.cells);
                }
            } else {
                row = Self::build_row(
                    fonts,
                    page_height,
                    table,
                    &table.columns,
                    item,
                    widths,
                    positions,
                    current_y,
                    ctx.clone(),
                );
            }
            current_y += row.height;
            rows.push(row);
        }

        rows
    }

    fn build_row(
        fonts: &PdfFonts,
        page_height: f32,
        table: &TableElement,
        columns: &[TableColumn],
        data: &Value,
        widths: &[f32],
        positions: &[f32],
        y: f32,
        ctx: FormatterContext,
    ) -> TableRowLayout {
        let mut row = TableRowLayout {
            y,

            height: DEFAULT_ROW_HEIGHT,

            cells: Vec::new(),
        };

        let mut index = 0;
        for column in columns {
            if index >= widths.len() || index >= positions.len() {
                break;
            }
            let span = column.col_span.max(1).min(widths.len() - index);
            let mut cell = Self::build_cell(
                column,
                data,
                positions[index],
                y,
                TableLayoutEngine::span_width(widths, index, span),
                &table.style,
                ctx.clone(),
            );
            cell.col_span = span;
            row.cells.push(cell);
            index += span;
        }
        let row_height = Self::measure_row_height(fonts, page_height, &row.cells, ctx.clone());
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
        ctx: FormatterContext,
    ) -> TableCellLayout {
        let mut style = TableLayoutEngine::merge_style(table_style, &column.body_style);

        if let Some(class_name) = data.get("$className").and_then(|v| v.as_str()) {
            if class_name != "" {
                class_name_to_style(class_name, &mut style);
                // println!("class_name={} new style>>{:#?}", class_name, style);
            }
        }

        let value = if column.field_name.trim().is_empty() {
            column.content.clone().unwrap_or_default()
        } else {
            resolve_value(data, &column.field_name)
                .map(|v| {
                    if v.is_string() {
                        v.as_str().unwrap().to_string()
                    } else {
                        v.to_string()
                    }
                })
                .unwrap_or_default()
        };

        let format_string = column.format_string.clone();

        TableCellLayout {
            x,
            y,
            width,
            height: 0.0,
            row_span: 1,
            col_span: 1,
            content: Self::apply_format(ctx, value, &format_string),
            style,
            is_row: true,
        }
    }

    pub fn apply_format(
        ctx: FormatterContext,
        value: String,
        format_string: &Option<String>,
    ) -> String {
        let Some(format) = format_string.as_deref() else {
            return value;
        };
        if format.trim().is_empty() {
            return value;
        }

        let (formatter, args) = match format {
            "SLG" | "GIA_NT" | "GIA" | "TIEN_NT" | "TIEN" | "EXCHANGE_RATE" | "PT" => (
                "formatNumber",
                vec![
                    Value::from(value.parse::<f64>().unwrap_or(0.0)),
                    Value::String(format.to_string()),
                ],
            ),
            _ => (
                "formatDate",
                vec![Value::String(value), Value::String(format.to_string())],
            ),
        };

        FORMATTERS.call(&ctx, formatter, &args).unwrap_or_default()
    }

    fn measure_row_height(
        fonts: &PdfFonts,
        page_height: f32,
        cells: &[TableCellLayout],
        ctx: FormatterContext,
    ) -> f32 {
        let mut max_height = DEFAULT_ROW_HEIGHT;

        for cell in cells {
            let text = TextElement {
                name: None,
                x: 0.0,
                y: 0.0,
                // Match TableRenderer's horizontal inset when wrapping text.
                width: (cell.width - 4.0).max(0.0),
                height: 0.0,
                content: cell.content.clone(),
                field_name: None,
                style: Some(cell.style.clone()),
                auto_height: Some(true),
                visible_if: None,
            };

            let layout = TextLayout::layout(
                fonts,
                page_height,
                &text,
                &serde_json::json!({}),
                ctx.clone(),
            );

            // padding trên + dưới
            let cell_height = layout.height + 3.0;

            max_height = max_height.max(cell_height);
        }

        max_height
    }
}
