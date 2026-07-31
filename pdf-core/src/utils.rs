use crate::fonts::{PdfFont, PdfFonts};
use crate::layout::TextLayout;
use crate::state::{FONT_BOLD, FONT_BOLD_ITALIC, FONT_ITALIC, FONT_REGULAR};
use crate::template::models::FormatterContext;
use anyhow::{anyhow, Result};
use printpdf::{
    Color, Greyscale, Mm, Op, ParsedFont, PdfDocument, Point, Pt, Rgb, TextItem, TextMatrix,
};
use regex::Regex;
use serde_json::Value;
use ttf_parser::Face;

pub struct Unit;

impl Unit {
    pub const DPI: f32 = 96.0;

    #[inline]
    pub fn px_to_mm(px: f32) -> Mm {
        Mm(px * 25.4 / Self::DPI)
    }

    #[inline(always)]
    pub fn px100_to_mm(px100: i32) -> Mm {
        Mm(px100 as f32 * 25.4 / (96.0 * 100.0))
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

    // BoldItalic
    let bold_italic = ParsedFont::from_bytes(FONT_BOLD_ITALIC, 0, &mut warnings)
        .ok_or_else(|| anyhow!("Cannot parse bold italic font"))?;
    let bold_italic_id = pdf.add_font(&bold_italic);

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
        bold_italic: PdfFont {
            id: bold_italic_id,
            parsed: bold_italic,
            bytes: FONT_BOLD_ITALIC,
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
use crate::border::Border;
use crate::models::ElementStyle;
pub fn draw_element_border(
    ops: &mut Vec<Op>,
    fonts: &PdfFonts,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    style: &ElementStyle,
) {
    Border::draw_rect(
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

pub fn draw_rect(
    ops: &mut Vec<Op>,
    fonts: &PdfFonts,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    style: &ElementStyle,
) {
    Border::draw_rect(
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

pub fn draw_circle(
    ops: &mut Vec<Op>,
    fonts: &PdfFonts,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    style: &ElementStyle,
) {
    Border::draw_rect(
        ops,
        fonts,
        x,
        y,
        width,
        height,
        width,
        style.background_color.as_deref(),
        style.border_color.as_deref(),
        Some(style.border_width.unwrap_or(0.0)),
        style.border_style.as_deref(),
    );
}

pub fn draw_line(
    ops: &mut Vec<Op>,
    fonts: &PdfFonts,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    style: &ElementStyle,
) {
    Border::draw_line(
        ops,
        fonts,
        x,
        y,
        width,
        height,
        style.background_color.as_deref(),
        style.border_style.as_deref(),
    );
}

pub fn get_formatter_context(data: &Value) -> FormatterContext {
    if let Some(config) = data.get("config") {
        serde_json::from_value(config.clone()).unwrap_or_default()
    } else {
        FormatterContext::default()
    }
}

pub fn draw_watermark(
    ops: &mut Vec<Op>,
    fonts: &PdfFonts,
    page_width: f32,
    page_height: f32,
    text: &str,
) {
    let target_width = page_width.min(page_height) * 0.80;

    let base_font_size = 50.0;

    let base_width = TextLayout::measure_string(fonts, text, base_font_size, true, false);

    if base_width <= 0.0 {
        return;
    }

    let font_size = base_font_size * target_width / base_width;

    let face = match Face::parse(fonts.bold.bytes, 0) {
        Ok(face) => face,
        Err(_) => return,
    };

    let (text_width, min_y, max_y) = TextLayout::measure_text_bbox(&face, text, font_size);

    let angle = 54.0_f32.to_radians();

    let cos = angle.cos();
    let sin = angle.sin();

    let cx = page_width / 2.0;
    let cy = page_height / 2.0;

    /*
     * Text origin của PDF nằm tại baseline.
     *
     * Bounding box:
     *
     *       max_y
     *          |
     *     ┌───────────┐
     *     │   TEXT    │
     *     └───────────┘
     *          |
     *       min_y
     *
     * Tâm bbox theo Y:
     */
    let bbox_center_y = (min_y + max_y) / 2.0;

    /*
     * Tâm bbox theo X.
     *
     * Vì width đang tính theo advance width,
     * dùng 1/2 width.
     */
    let bbox_center_x = text_width / 2.0;

    /*
     * Vector từ text origin tới tâm bbox.
     */
    let offset_x = bbox_center_x;
    let offset_y = bbox_center_y;

    /*
     * Rotate vector.
     */
    let rotated_x = offset_x * cos - offset_y * sin;

    let rotated_y = offset_x * sin + offset_y * cos;

    /*
     * Đưa tâm bbox vào đúng tâm trang.
     */
    let origin_x = cx - rotated_x;
    let origin_y = cy - rotated_y;

    let x_pt = Unit::px_to_mm(origin_x).into_pt().0;

    let y_pt = Unit::px_to_mm(origin_y).into_pt().0;

    ops.push(Op::StartTextSection);

    ops.push(Op::SetFillColor {
        col: Color::Greyscale(Greyscale::new(0.90, None)),
    });

    ops.push(Op::SetFontSize {
        font: fonts.bold.id.clone(),
        size: Unit::px_to_pt(font_size),
    });

    ops.push(Op::SetTextMatrix {
        matrix: TextMatrix::Raw([cos, sin, -sin, cos, x_pt, y_pt]),
    });

    ops.push(Op::WriteText {
        items: vec![TextItem::Text(text.to_string())],
        font: fonts.bold.id.clone(),
    });

    ops.push(Op::EndTextSection);
}
