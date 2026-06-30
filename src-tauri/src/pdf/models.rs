use serde::Deserialize;

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
}

#[derive(Debug, Deserialize)]
pub struct TextElement {
    pub x: f32,

    pub y: f32,

    #[serde(rename = "fontSize")]
    pub font_size: f32,

    pub value: String,
}
