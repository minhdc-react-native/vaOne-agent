use crate::table::models::TableElement;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
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
    pub visible: Option<bool>,
}
impl TextLayoutResult {
    pub fn translate_y(&mut self, dy: f32) {
        self.y += dy;
    }
}
#[derive(Debug, Deserialize, Clone)]
pub struct PdfTemplate {
    pub name: String,
    pub width: f32,
    pub height: f32,
    #[serde(rename = "backgroundImage")]
    pub background_image: Option<String>,
    pub elements: Vec<Element>,
}

#[derive(Debug, Deserialize)]
pub struct Page {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Deserialize, Clone)]
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

    #[serde(rename = "image")]
    Image(LRCElement),

    #[serde(rename = "grid")]
    Grid(GridElement),
}

impl Element {
    pub fn visible_if(&self) -> Option<String> {
        match self {
            Element::Text(e) => e.visible_if.clone(),
            Element::Table(e) => e.visible_if.clone(),
            Element::Line(e) => e.visible_if.clone(),
            Element::Rect(e) => e.visible_if.clone(),
            Element::Circle(e) => e.visible_if.clone(),
            Element::Image(e) => e.visible_if.clone(),
            Element::Grid(e) => e.visible_if.clone(),
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Element::Text(e) => e.name.as_deref(),
            Element::Table(e) => e.name.as_deref(),
            Element::Line(e) => e.name.as_deref(),
            Element::Rect(e) => e.name.as_deref(),
            Element::Circle(e) => e.name.as_deref(),
            Element::Image(e) => e.name.as_deref(),
            Element::Grid(e) => e.name.as_deref(),
        }
    }

    pub fn as_text(&self) -> Option<&TextElement> {
        match self {
            Element::Text(e) => Some(e),
            _ => None,
        }
    }

    pub fn x(&self) -> f32 {
        match self {
            Element::Text(e) => e.x,
            Element::Table(e) => e.x,
            Element::Line(e) => e.x,
            Element::Rect(e) => e.x,
            Element::Circle(e) => e.x,
            Element::Image(e) => e.x,
            Element::Grid(e) => e.x,
        }
    }

    pub fn y(&self) -> f32 {
        match self {
            Element::Text(e) => e.y,
            Element::Table(e) => e.y,
            Element::Line(e) => e.y,
            Element::Rect(e) => e.y,
            Element::Circle(e) => e.y,
            Element::Image(e) => e.y,
            Element::Grid(e) => e.y,
        }
    }

    pub fn height(&self) -> f32 {
        match self {
            Element::Text(e) => e.height,
            Element::Table(e) => e.height,
            Element::Line(e) => e.height,
            Element::Rect(e) => e.height,
            Element::Circle(e) => e.height,
            Element::Image(e) => e.height,
            Element::Grid(e) => e.height,
        }
    }
}

pub trait ElementVecExt {
    fn extract_page_number(self) -> (Option<Element>, Vec<Element>);
    fn sort_by_y(&mut self);
}

impl ElementVecExt for Vec<Element> {
    fn extract_page_number(self) -> (Option<Element>, Vec<Element>) {
        let mut page_number = None;

        let elements = self
            .into_iter()
            .filter_map(|e| {
                if e.name() == Some("pageNumber") {
                    page_number = Some(e);
                    None
                } else {
                    Some(e)
                }
            })
            .collect();

        (page_number, elements)
    }

    fn sort_by_y(&mut self) {
        self.sort_by(|a, b| a.y().total_cmp(&b.y()));
    }
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
    #[serde(rename = "textDecoration")]
    pub text_decoration: Option<String>,

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
            text_decoration: Some("none".to_string()),
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

    #[serde(rename = "visibleIf")]
    pub visible_if: Option<String>,

    #[serde(rename = "fieldName")]
    pub field_name: Option<String>,

    #[serde(rename = "autoHeight")]
    pub auto_height: Option<bool>,

    #[serde(default)]
    pub style: Option<ElementStyle>,
}

impl TextElement {
    pub fn translate_y(&mut self, dy: f32) {
        self.y += dy;
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LRCElement {
    pub name: Option<String>,
    pub x: f32,
    pub y: f32,

    #[serde(rename = "fieldName")]
    pub field_name: Option<String>,

    pub content: Option<String>,

    pub width: f32,
    pub height: f32,

    #[serde(rename = "visibleIf")]
    pub visible_if: Option<String>,

    #[serde(default)]
    pub style: Option<ElementStyle>,

    pub visible: Option<bool>,
}
impl LRCElement {
    pub fn translate_y(&mut self, dy: f32) {
        self.y += dy;
    }
}
#[derive(Debug, Clone, Deserialize)]
pub struct GridElement {
    pub name: Option<String>,
    pub x: f32,
    pub y: f32,

    pub width: f32,
    pub height: f32,

    pub content: Option<String>,

    #[serde(rename = "fieldName")]
    pub field_name: Option<String>,

    #[serde(rename = "visibleIf")]
    pub visible_if: Option<String>,

    #[serde(default)]
    pub style: Option<ElementStyle>,

    pub visible: Option<bool>,
}
impl GridElement {
    pub fn translate_y(&mut self, dy: f32) {
        self.y += dy;
    }
}
