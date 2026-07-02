use serde::Deserialize;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RichStyle {
    Normal,
    B,
    I,
    U,
    BI,
    BU,
    IU,
    BIU,
}

#[derive(Debug, Clone)]
pub struct TextRun {
    pub text: String,

    pub style: RichStyle,

    pub color: Option<String>,

    pub size: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct TextLine {
    /// Nội dung của dòng
    pub text: String,

    /// Chiều rộng thực tế của dòng (mm)
    pub width: f32,
}
#[derive(Debug, Clone)]
pub struct TextLayoutResult {
    pub lines: Vec<TextLine>,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub line_height: f32,
    pub base_y: f32,
}

#[derive(Debug, Deserialize)]
pub struct PdfTemplate {
    pub page: Page,
    pub elements: Vec<Element>,
}

#[derive(Debug, Deserialize)]
pub struct Page {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Element {
    #[serde(rename = "text")]
    Text(TextElement),

    #[serde(rename = "table")]
    Table(TableElement),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ElementStyle {
    #[serde(default)]
    #[serde(rename = "backgroundColor")]
    pub background_color: Option<String>,

    #[serde(default)]
    pub opacity: Option<f32>,

    #[serde(default)]
    #[serde(rename = "fontSize")]
    pub font_size: Option<f32>,

    #[serde(default)]
    pub color: Option<String>,

    #[serde(default)]
    #[serde(rename = "textAlign")]
    pub text_align: Option<String>,

    #[serde(default)]
    #[serde(rename = "fontWeight")]
    pub font_weight: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextElement {
    pub x: f32,
    pub y: f32,

    pub width: f32,
    pub height: f32,

    pub content: String,

    #[serde(rename = "fieldName")]
    pub field_name: Option<String>,

    #[serde(default)]
    pub style: Option<ElementStyle>,
}
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum TableWidth {
    Px(f32),
    Auto(String), // "auto"
}

#[derive(Debug, Deserialize, Clone)]
pub struct TableColumn {
    pub id: String,
    pub header: String,
    pub field_name: String,

    /// px hoặc auto (string để giữ flexibility)
    pub width: TableWidth,

    pub content: Option<String>,
    pub format_string: Option<String>,

    pub header_style: Option<ElementStyle>,
    pub body_style: Option<ElementStyle>,

    pub col_span: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct TableElement {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<serde_json::Value>,
    pub data_field: Option<String>, // ví dụ: "orders"
}
