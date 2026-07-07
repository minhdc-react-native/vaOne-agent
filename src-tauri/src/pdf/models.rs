use crate::pdf::table::models::TableElement;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<String>,
    pub font_size: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct TextRun {
    pub text: String,

    pub style: TextStyle,

    pub color: Option<String>,

    pub size: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct TextLine {
    pub runs: Vec<TextRun>,
    pub width: f32,
}
#[derive(Debug, Clone)]
pub struct TextLayoutResult {
    pub lines: Vec<TextLine>,
    pub content_width: f32,
    pub content_height: f32,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub line_height: f32,
    pub base_y: f32,
}
#[derive(Debug, Deserialize)]
pub struct PdfTemplate {
    pub width: f32,
    pub height: f32,
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

    #[serde(rename = "line")]
    Line(LRCElement),

    #[serde(rename = "rect")]
    Rect(LRCElement),

    #[serde(rename = "circle")]
    Circle(LRCElement),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub center_y: Option<bool>,

    #[serde(default)]
    #[serde(rename = "fontWeight")]
    pub font_weight: Option<String>,

    #[serde(default)]
    #[serde(rename = "fontStyle")]
    pub font_style: Option<String>,

    #[serde(default)]
    #[serde(rename = "borderColor")]
    pub border_color: Option<String>,

    #[serde(default)]
    #[serde(rename = "borderRadius")]
    pub border_radius: Option<f32>,

    #[serde(default)]
    #[serde(rename = "borderWidth")]
    pub border_width: Option<f32>,

    #[serde(default)]
    #[serde(rename = "borderStyle")]
    pub border_style: Option<String>,

    #[serde(default)]
    pub padding: Option<f32>,

    #[serde(default)]
    #[serde(rename = "fontFamily")]
    pub font_family: Option<String>,

    #[serde(default)]
    #[serde(rename = "marginTop")]
    pub margin_top: Option<f32>,

    #[serde(default)]
    #[serde(rename = "marginRight")]
    pub margin_right: Option<f32>,

    #[serde(default)]
    #[serde(rename = "marginBottom")]
    pub margin_bottom: Option<f32>,

    #[serde(default)]
    #[serde(rename = "marginLeft")]
    pub margin_left: Option<f32>,
}

impl Default for ElementStyle {
    fn default() -> Self {
        Self {
            background_color: Some("transparent".to_string()),
            opacity: Some(1.0),
            font_size: Some(14.0),
            color: Some("#000000".to_string()),
            text_align: Some("left".to_string()),
            center_y: Some(false),
            font_weight: Some("normal".to_string()),
            font_style: Some("normal".to_string()),
            border_color: None,
            border_radius: None,
            border_width: None,
            border_style: None,
            padding: None,
            font_family: None,
            margin_top: None,
            margin_right: None,
            margin_left: None,
            margin_bottom: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextElement {
    pub x: f32,
    pub y: f32,

    pub width: f32,
    pub height: f32,

    pub content: String,
    pub name: Option<String>,

    #[serde(rename = "fieldName")]
    pub field_name: Option<String>,

    #[serde(rename = "autoHeight")]
    pub auto_height: Option<bool>,

    #[serde(default)]
    pub style: Option<ElementStyle>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LRCElement {
    pub name: Option<String>,
    pub x: f32,
    pub y: f32,

    pub width: f32,
    pub height: f32,

    #[serde(default)]
    pub style: Option<ElementStyle>,
}
