use crate::pdf::models::{ElementStyle, TextRun};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnWidth {
    Fixed(f32),
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableElement {
    pub name: Option<String>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,

    #[serde(default)]
    #[serde(rename = "fieldName")]
    pub field_name: Option<String>,

    #[serde(default)]
    pub columns: Vec<TableColumn>,

    #[serde(default)]
    #[serde(rename = "headerLayout")]
    pub header_layout: Vec<Vec<TableHeaderCell>>,

    #[serde(default)]
    #[serde(rename = "childElements")]
    pub child_elements: Vec<TableChildElement>,

    #[serde(default)]
    #[serde(rename = "fixRow")]
    pub fix_row: Option<TableFixRow>,

    #[serde(default)]
    pub style: Option<ElementStyle>,
}
impl TableElement {
    pub fn translate_y(&mut self, dy: f32) {
        self.y += dy;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub id: String,

    pub header: String,

    #[serde(rename = "fieldName")]
    pub field_name: String,

    pub content: Option<String>,

    #[serde(rename = "formatString")]
    pub format_string: Option<String>,

    #[serde(default)]
    pub width: serde_json::Value,

    #[serde(default)]
    #[serde(rename = "headerStyle")]
    pub header_style: Option<ElementStyle>,

    #[serde(default)]
    #[serde(rename = "bodyStyle")]
    pub body_style: Option<ElementStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableHeaderCell {
    pub content: String,

    #[serde(default)]
    #[serde(rename = "rowSpan")]
    pub row_span: usize,

    #[serde(default)]
    #[serde(rename = "colSpan")]
    pub col_span: usize,

    #[serde(default)]
    pub style: Option<ElementStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableFixRow {
    pub row: usize,

    #[serde(default)]
    pub data: Vec<TableRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    #[serde(default)]
    pub columns: Vec<TableColumn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableChildElement {}

#[derive(Debug, Clone)]
pub struct TableLayoutResult {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub headers: Vec<TableRowLayout>,
    pub rows: Vec<TableRowLayout>,
}
impl TableLayoutResult {
    /// Tổng chiều cao phần header
    pub fn header_height(&self) -> f32 {
        self.headers
            .iter()
            .map(|row| {
                row.cells
                    .iter()
                    .map(|cell| cell.height)
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0)
            })
            .sum()
    }

    /// Tổng chiều cao phần body
    pub fn rows_height(&self) -> f32 {
        self.rows.iter().map(|r| r.height).sum()
    }

    /// Tính lại chiều cao toàn bộ table
    pub fn recalc_height(&mut self) {
        self.height = self.header_height() + self.rows_height();
    }

    /// Dịch chuyển toàn bộ table theo trục Y
    pub fn translate_y(&mut self, dy: f32) {
        self.y += dy;

        for header in &mut self.headers {
            header.translate_y(dy);
        }

        for row in &mut self.rows {
            row.translate_y(dy);
        }
    }
    pub fn clone_headers(&self) -> Vec<TableRowLayout> {
        self.headers.clone()
    }

    pub fn empty_body(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.header_height(),
            headers: self.headers.clone(),
            rows: Vec::new(),
        }
    }

    pub fn push_row(&mut self, mut row: TableRowLayout) {
        let y = if let Some(last) = self.rows.last() {
            last.y + last.height
        } else if let Some(last_header) = self.headers.last() {
            last_header.y + last_header.height
        } else {
            self.y
        };

        row.translate_y(y - row.y);

        self.rows.push(row);

        self.recalc_height();
    }
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }
}
#[derive(Debug, Clone)]
pub struct TableRowLayout {
    pub y: f32,
    pub height: f32,

    pub cells: Vec<TableCellLayout>,
}

impl TableRowLayout {
    pub fn translate_y(&mut self, dy: f32) {
        self.y += dy;

        for cell in &mut self.cells {
            cell.y += dy;
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TableCellLayout {
    pub x: f32,
    pub y: f32,

    pub width: f32,
    pub height: f32,

    // pub runs: Vec<TextRun>,
    pub content: String,

    pub style: ElementStyle,

    pub row_span: usize,
    pub col_span: usize,
    pub is_row: bool,
}
