use crate::pdf::fonts::{PdfFont, PdfFonts};
use anyhow::{anyhow, Result};
use printpdf::{Color, Rgb};
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

pub fn resolve_array(data: &serde_json::Value, path: &str) -> Option<Vec<serde_json::Value>> {
    let mut current = data;

    for key in path.split('.') {
        current = current.get(key)?;
    }

    match current {
        serde_json::Value::Array(arr) => Some(arr.clone()),
        _ => None,
    }
}

pub fn resolve_value(data: &Value, path: &str) -> Option<String> {
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

    match current {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

pub fn bind_content(template: &str, data: &Value) -> String {
    let re = Regex::new(r"\{([^{}]+)\}").unwrap();

    re.replace_all(template, |caps: &regex::Captures| {
        resolve_value(data, &caps[1]).unwrap_or_default()
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
