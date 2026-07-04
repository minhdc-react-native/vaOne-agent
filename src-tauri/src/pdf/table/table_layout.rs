use super::super::models::{ElementStyle, TextRun};
use super::models::{
    TableCellLayout, TableColumn, TableElement, TableLayoutResult, TableRowLayout,
};
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
    pub fn build(table: &TableElement) -> TableLayoutResult {
        let widths = Self::calc_column_widths(table);

        let positions = Self::calc_column_positions(table.x, &widths);

        let mut rows = Vec::new();

        //----------------------------------------------------
        // Header
        //----------------------------------------------------

        let header = Self::build_header_rows(table, &widths, &positions);

        rows.extend(header);

        //----------------------------------------------------
        // FixRow
        //----------------------------------------------------

        let fix = Self::build_fix_rows(table, &widths, &positions, rows.len());

        rows.extend(fix);

        //----------------------------------------------------

        let height = rows.iter().map(|r| r.height).sum();

        TableLayoutResult {
            width: table.width,

            height,

            rows,
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

    pub fn merge_style(table: &Option<ElementStyle>, cell: &Option<ElementStyle>) -> ElementStyle {
        let mut style = table.clone().unwrap_or_default();

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

        let mut result = Vec::new();

        //------------------------------------------------------
        // mỗi cột còn bị rowSpan chiếm bao nhiêu dòng
        //------------------------------------------------------

        let mut occupied = vec![0usize; table.columns.len()];

        //------------------------------------------------------

        for (row_index, row) in table.header_layout.iter().enumerate() {
            //--------------------------------------------------
            // sang dòng mới
            //--------------------------------------------------

            for c in occupied.iter_mut() {
                if *c > 0 {
                    *c -= 1;
                }
            }

            let mut column = 0usize;

            for cell in row {
                //------------------------------
                // tìm cột còn trống
                //------------------------------

                while column < occupied.len() && occupied[column] > 0 {
                    column += 1;
                }

                if column >= widths.len() {
                    break;
                }

                let col_span = cell.col_span.max(1);
                let row_span = cell.row_span.max(1);

                //------------------------------
                // width
                //------------------------------

                let width = TableLayoutEngine::span_width(widths, column, col_span);

                //------------------------------
                // height
                //------------------------------

                let height = row_height * row_span as f32;

                //------------------------------
                // style
                //------------------------------

                let style = TableLayoutEngine::merge_style(&table.style, &cell.style);

                //------------------------------
                // runs
                //------------------------------

                let runs = TableLayoutEngine::build_runs(cell.content.clone());

                //------------------------------
                // tạo layout
                //------------------------------

                result.push(TableCellLayout {
                    x: positions[column],

                    y: table.y + row_index as f32 * row_height,

                    width,

                    height,

                    runs,

                    style,

                    row_span,

                    col_span,
                });

                //------------------------------
                // đánh dấu rowSpan
                //------------------------------

                if row_span > 1 {
                    for c in column..column + col_span {
                        if c < occupied.len() {
                            occupied[c] = row_span - 1;
                        }
                    }
                }

                //------------------------------
                // sang cột tiếp
                //------------------------------

                column += col_span;
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

        for row_index in 0..table.header_layout.len() {
            let y = table.y + row_index as f32 * DEFAULT_HEADER_HEIGHT;

            let mut row = TableRowLayout {
                y,

                height: DEFAULT_HEADER_HEIGHT,

                cells: Vec::new(),
            };

            for cell in &cells {
                if ((cell.y - y).abs()) < 0.01 {
                    row.cells.push(cell.clone());
                }
            }

            rows.push(row);
        }

        rows
    }

    fn build_fix_rows(
        table: &TableElement,

        widths: &[f32],

        positions: &[f32],

        start_row: usize,
    ) -> Vec<TableRowLayout> {
        let mut rows = Vec::new();

        let Some(fix) = &table.fix_row else {
            return rows;
        };

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

                let style = Self::merge_style(&table.style, &col.body_style);

                row.cells.push(TableCellLayout {
                    x,

                    y,

                    width,

                    height: DEFAULT_HEADER_HEIGHT,

                    runs: Self::build_runs(col.header.clone()),

                    style,

                    row_span: 1,

                    col_span: 1,
                });

                x += width;
            }

            rows.push(row);
        }

        rows
    }
}
