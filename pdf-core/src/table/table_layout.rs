use crate::fonts::PdfFonts;
use crate::models::{ElementStyle, TextRun};
use crate::table::models::{
    TableCellLayout, TableElement, TableHeaderCell, TableLayoutResult, TableRowLayout,
};
use crate::table::table_row::TableRow;
use crate::template::models::FormatterContext;
use crate::utils::{get_formatter_context, resolve_array_table, resolve_value};
use serde_json::Value;
pub struct TableLayoutEngine;

const DEFAULT_HEADER_HEIGHT: f32 = 24.0;

impl TableLayoutEngine {
    /// Build layout table.
    ///
    /// Phần này mới chỉ:
    /// - tính width các cột
    /// - tính vị trí X
    /// - khởi tạo các dòng header
    ///
    /// Chưa sinh CellLayout.
    pub fn build(
        fonts: &PdfFonts,
        page_height: f32,
        table: &TableElement,
        data: &Value,
        ctx: FormatterContext,
    ) -> TableLayoutResult {
        let widths = Self::calc_column_widths(table);

        let positions = Self::calc_column_positions(table.x, &widths);

        let mut headers = Vec::new();
        let mut rows = Vec::new();

        //----------------------------------------------------
        // Header
        //----------------------------------------------------

        if !table.field_name.as_deref().unwrap_or("").trim().is_empty() {
            let header = Self::build_header_rows(table, &widths, &positions);
            headers.extend(header.clone());
        } else {
            let fix = Self::build_fix_rows(table, &widths, &positions, rows.len(), data);
            rows.extend(fix);
        }

        //----------------------------------------------------
        // FixRow
        //----------------------------------------------------

        //----------------------------------------------------

        let header_height: f32 = headers
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|cell| cell.height)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0)
            })
            .sum();

        let field = table
            .field_name
            .as_deref()
            .expect("Table field_name is required");

        let context = resolve_array_table(&data, field);

        let body = TableRow::build_rows(
            fonts,
            page_height,
            table,
            &widths,
            &positions,
            table.y + header_height,
            context,
            ctx,
        );
        rows.extend(body);

        let height = header_height + rows.iter().map(|r| r.height).sum::<f32>();

        TableLayoutResult {
            x: table.x,
            y: table.y,
            width: table.width,

            height,
            headers,
            rows,
            visible: Some(true),
        }
    }

    //------------------------------------------------------------------
    // Width
    //------------------------------------------------------------------

    pub fn calc_column_widths(table: &TableElement) -> Vec<f32> {
        let mut result = Vec::new();

        let mut fixed = 0f32;

        let mut auto = Vec::<usize>::new();

        for (idx, col) in table.columns.iter().enumerate() {
            match Self::parse_width(&col.width) {
                Some(v) => {
                    result.push(v);
                    fixed += v;
                }
                None => {
                    result.push(0.0);
                    auto.push(idx);
                }
            }
        }

        if !auto.is_empty() {
            let remain = (table.width - fixed).max(0.0);

            let auto_width = remain / auto.len() as f32;

            for idx in auto {
                result[idx] = auto_width;
            }
        }

        result
    }

    //------------------------------------------------------------------
    // Column X
    //------------------------------------------------------------------

    pub fn calc_column_positions(start_x: f32, widths: &[f32]) -> Vec<f32> {
        let mut result = Vec::with_capacity(widths.len());

        let mut x = start_x;

        for w in widths {
            result.push(x);
            x += *w;
        }

        result
    }

    //------------------------------------------------------------------
    // Width helper
    //------------------------------------------------------------------

    fn parse_width(value: &Value) -> Option<f32> {
        match value {
            Value::Number(v) => v.as_f64().map(|v| v as f32),

            Value::String(s) => {
                let s = s.trim().to_lowercase();

                if s == "auto" {
                    return None;
                }

                if s.ends_with("px") {
                    return s.replace("px", "").parse::<f32>().ok();
                }

                s.parse::<f32>().ok()
            }

            _ => None,
        }
    }

    //------------------------------------------------------------------
    // Span helper
    //------------------------------------------------------------------

    pub fn span_width(widths: &[f32], column: usize, span: usize) -> f32 {
        widths.iter().skip(column).take(span).sum()
    }

    //------------------------------------------------------------------
    // Merge Style
    //------------------------------------------------------------------

    pub fn merge_style(
        element_style: &Option<ElementStyle>,
        cell: &Option<ElementStyle>,
    ) -> ElementStyle {
        let mut style = element_style.clone().unwrap_or_default();

        if let Some(c) = cell {
            if c.background_color.is_some() {
                style.background_color = c.background_color.clone();
            }

            if c.color.is_some() {
                style.color = c.color.clone();
            }

            if c.font_family.is_some() {
                style.font_family = c.font_family.clone();
            }

            if c.font_size.is_some() {
                style.font_size = c.font_size;
            }

            if c.font_weight.is_some() {
                style.font_weight = c.font_weight.clone();
            }

            if c.font_style.is_some() {
                style.font_style = c.font_style.clone();
            }

            if c.text_align.is_some() {
                style.text_align = c.text_align.clone();
            }

            if c.border_color.is_some() {
                style.border_color = c.border_color.clone();
            }

            if c.border_width.is_some() {
                style.border_width = c.border_width;
            }

            if c.border_style.is_some() {
                style.border_style = c.border_style.clone();
            }

            if c.padding.is_some() {
                style.padding = c.padding;
            }

            if c.opacity.is_some() {
                style.opacity = c.opacity;
            }
        }

        style
    }

    //------------------------------------------------------------------
    // TextRun
    //------------------------------------------------------------------

    pub fn build_runs(text: impl Into<String>) -> Vec<TextRun> {
        vec![TextRun {
            text: text.into(),
            style: Default::default(),
            color: None,
            size: None,
        }]
    }

    pub fn build_header_cells(
        table: &TableElement,
        widths: &[f32],
        positions: &[f32],
    ) -> Vec<TableCellLayout> {
        let row_height = DEFAULT_HEADER_HEIGHT;

        //--------------------------------------------------
        // Header rows
        //--------------------------------------------------

        let header_rows: Vec<Vec<TableHeaderCell>> = if table.header_layout.is_empty() {
            vec![table
                .columns
                .iter()
                .map(|col| TableHeaderCell {
                    content: col.header.clone(),
                    col_span: 1,
                    row_span: 1,
                    style: col.header_style.clone(),
                })
                .collect()]
        } else {
            table.header_layout.clone()
        };

        let row_count = header_rows.len();
        let col_count = table.columns.len();

        //--------------------------------------------------
        // Grid đánh dấu các ô đã bị chiếm
        //--------------------------------------------------

        let mut grid = vec![vec![false; col_count]; row_count];

        let mut result = Vec::new();

        //--------------------------------------------------

        for (row_index, row) in header_rows.iter().enumerate() {
            let mut search_col = 0usize;

            for cell in row {
                let col_span = cell.col_span.max(1);
                let row_span = cell.row_span.max(1);

                //------------------------------------------
                // tìm cột đầu tiên còn trống
                //------------------------------------------

                while search_col < col_count && grid[row_index][search_col] {
                    search_col += 1;
                }

                if search_col >= col_count {
                    break;
                }

                //------------------------------------------
                // đánh dấu toàn bộ vùng rowspan/colspan
                //------------------------------------------

                for r in row_index..(row_index + row_span).min(row_count) {
                    for c in search_col..(search_col + col_span).min(col_count) {
                        grid[r][c] = true;
                    }
                }

                //------------------------------------------
                // width
                //------------------------------------------

                let width = Self::span_width(widths, search_col, col_span);

                //------------------------------------------
                // height
                //------------------------------------------

                let height = row_height * row_span as f32;

                //------------------------------------------
                // style
                //------------------------------------------

                let mut style = Self::merge_style(&table.style, &cell.style);
                style.center_y = Some(true);

                //------------------------------------------
                // push
                //------------------------------------------

                result.push(TableCellLayout {
                    x: positions[search_col],
                    y: table.y + row_index as f32 * row_height,
                    width,
                    height,
                    content: cell.content.clone(),
                    style,
                    row_span,
                    col_span,
                    is_row: false,
                });

                //------------------------------------------
                // tìm tiếp
                //------------------------------------------

                search_col += col_span;
            }
        }

        result
    }

    fn build_header_rows(
        table: &TableElement,

        widths: &[f32],

        positions: &[f32],
    ) -> Vec<TableRowLayout> {
        let cells = Self::build_header_cells(table, widths, positions);

        let mut rows = Vec::new();

        let row_count = if table.header_layout.is_empty() {
            1
        } else {
            table.header_layout.len()
        };

        for row_index in 0..row_count {
            let y = table.y + row_index as f32 * DEFAULT_HEADER_HEIGHT;

            let mut row = TableRowLayout {
                y,
                height: 0.0,
                cells: Vec::new(),
            };

            for cell in &cells {
                if ((cell.y - y).abs()) < 0.01 {
                    row.cells.push(cell.clone());
                }
            }

            row.height = row
                .cells
                .iter()
                .map(|c| c.height)
                .fold(DEFAULT_HEADER_HEIGHT, f32::max);

            rows.push(row);
        }

        rows
    }

    fn build_fix_rows(
        table: &TableElement,

        widths: &[f32],

        _positions: &[f32],

        start_row: usize,
        data: &Value,
    ) -> Vec<TableRowLayout> {
        let mut rows = Vec::new();

        let Some(fix) = &table.fix_row else {
            return rows;
        };
        let ctx: FormatterContext = get_formatter_context(data);
        for (r, row_cfg) in fix.data.iter().enumerate() {
            let y = table.y + (start_row + r) as f32 * DEFAULT_HEADER_HEIGHT;

            let mut row = TableRowLayout {
                y,

                height: DEFAULT_HEADER_HEIGHT,

                cells: Vec::new(),
            };

            let mut x = table.x;

            for (i, col) in row_cfg.columns.iter().enumerate() {
                let width = widths[i];

                let mut style = Self::merge_style(&table.style, &col.body_style);
                style.center_y = Some(true);

                let content = if table.field_name.as_deref().unwrap_or("").trim().is_empty() {
                    if col.field_name.trim().is_empty() {
                        col.content.clone().unwrap_or_default()
                    } else {
                        resolve_value(data, &col.field_name)
                            .map(|v| {
                                if let Some(s) = v.as_str() {
                                    s.to_string()
                                } else {
                                    v.to_string()
                                }
                            })
                            .unwrap_or_default()
                    }
                } else {
                    col.header.clone()
                };

                let format_string = col.format_string.clone();

                row.cells.push(TableCellLayout {
                    x,

                    y,

                    width,

                    height: DEFAULT_HEADER_HEIGHT,

                    content: TableRow::apply_format(ctx.clone(), content, &format_string),

                    style,

                    row_span: 1,

                    col_span: 1,
                    is_row: false,
                });

                x += width;
            }

            rows.push(row);
        }

        rows
    }
}
