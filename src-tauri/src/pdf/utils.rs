use crate::pdf::fonts::{PdfFont, PdfFonts};
use anyhow::{anyhow, Result};
use printpdf::{Color, Op, Rgb};
use printpdf::{ParsedFont, PdfDocument};

use crate::state::{FONT_BOLD, FONT_ITALIC, FONT_REGULAR};
use printpdf::{Mm, Pt};
use regex::Regex;
use serde_json::Value;
pub struct Unit;

impl Unit {
    pub const DPI: f32 = 96.0;

    #[inline]
    pub fn px_to_mm(px: f32) -> Mm {
        Mm(px * 25.4 / Self::DPI)
    }

    #[inline]
    pub fn mm_to_px(mm: f32) -> f32 {
        mm * Self::DPI / 25.4
    }

    #[inline]
    pub fn px_to_pt(px: f32) -> Pt {
        Pt(px * 72.0 / Self::DPI)
    }

    #[inline]
    pub fn pt_to_px(pt: f32) -> f32 {
        pt * Self::DPI / 72.0
    }
}

pub fn resolve_array_table<'a>(data: &'a Value, path: &str) -> &'a [Value] {
    let mut current = data;

    for key in path.split('.') {
        match current.get(key) {
            Some(value) => current = value,
            None => return &[],
        }
    }

    current.as_array().map(Vec::as_slice).unwrap_or(&[])
}

pub fn resolve_array<'a>(
    data: &'a serde_json::Value,
    path: &str,
) -> Option<&'a Vec<serde_json::Value>> {
    let mut current = data;

    for key in path.split('.') {
        current = current.get(key)?;
    }

    current.as_array()
}

pub fn resolve_value(data: &Value, path: &str) -> Option<Value> {
    let mut current = data;

    for key in path.split('.') {
        current = match current {
            Value::Object(map) => map.get(key)?,

            Value::Array(arr) => {
                let idx: usize = key.parse().ok()?;
                arr.get(idx)?
            }

            _ => return None,
        };
    }

    Some(current.clone())
}

pub fn bind_content(template: &str, data: &Value) -> String {
    if template.trim().is_empty() {
        return data
            .get("value")
            .map(|v| match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                other => other.to_string(),
            })
            .unwrap_or_default();
    }
    let re = Regex::new(r"\{([^{}]+)\}").unwrap();

    re.replace_all(template, |caps: &regex::Captures| {
        let value = resolve_value(data, &caps[1])
            .map(|v| match v {
                Value::String(s) => s,
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => String::new(),
                other => other.to_string(),
            })
            .unwrap_or_else(|| format!("{{{}}}", &caps[1]));
        return value;
    })
    .into_owned()
}

pub fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;

    Color::Rgb(Rgb::new(r, g, b, None))
}

pub fn load_fonts(pdf: &mut PdfDocument) -> Result<PdfFonts> {
    let mut warnings = Vec::new();

    // Regular
    let regular = ParsedFont::from_bytes(FONT_REGULAR, 0, &mut warnings)
        .ok_or_else(|| anyhow!("Cannot parse regular font"))?;
    let regular_id = pdf.add_font(&regular);

    // Bold
    let bold = ParsedFont::from_bytes(FONT_BOLD, 0, &mut warnings)
        .ok_or_else(|| anyhow!("Cannot parse bold font"))?;
    let bold_id = pdf.add_font(&bold);

    // Italic
    let italic = ParsedFont::from_bytes(FONT_ITALIC, 0, &mut warnings)
        .ok_or_else(|| anyhow!("Cannot parse italic font"))?;
    let italic_id = pdf.add_font(&italic);

    Ok(PdfFonts {
        regular: PdfFont {
            id: regular_id,
            parsed: regular,
            bytes: FONT_REGULAR,
        },
        bold: PdfFont {
            id: bold_id,
            parsed: bold,
            bytes: FONT_BOLD,
        },
        italic: PdfFont {
            id: italic_id,
            parsed: italic,
            bytes: FONT_ITALIC,
        },
    })
}

pub fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}
use crate::pdf::border::Border;
use crate::pdf::models::ElementStyle;
pub fn draw_element_border(
    ops: &mut Vec<Op>,
    fonts: &PdfFonts,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    style: &ElementStyle,
) {
    Border::draw(
        ops,
        fonts,
        x,
        y,
        width,
        height,
        style.border_radius.unwrap_or(0.0),
        style.background_color.as_deref(),
        style.border_color.as_deref(),
        Some(style.border_width.unwrap_or(0.0)),
        style.border_style.as_deref(),
    );
}
