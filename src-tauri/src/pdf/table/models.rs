use crate::pdf::models::{ElementStyle, TextRun};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnWidth {
    Fixed(f32),
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableElement {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub id: String,

    pub header: String,

    #[serde(rename = "fieldName")]
    pub field_name: String,

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

pub struct TableLayoutResult {
    pub width: f32,
    pub height: f32,

    pub rows: Vec<TableRowLayout>,
}

pub struct TableRowLayout {
    pub y: f32,
    pub height: f32,

    pub cells: Vec<TableCellLayout>,
}
#[derive(Debug, Clone)]
pub struct TableCellLayout {
    pub x: f32,
    pub y: f32,

    pub width: f32,
    pub height: f32,

    pub runs: Vec<TextRun>,

    pub style: ElementStyle,

    pub row_span: usize,
    pub col_span: usize,
}
