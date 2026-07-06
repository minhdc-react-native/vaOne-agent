use crate::pdf::fonts::PdfFonts;
use crate::pdf::models::{TextElement, TextLayoutResult, TextLine};
use crate::pdf::template::parser::Parser;
use crate::pdf::template::tokenizer::Tokenizer;
use crate::pdf::utils::{resolve_value, Unit};
use serde_json::{Map, Value};
use ttf_parser::Face;
pub struct TextLayout;

impl TextLayout {
    fn measure_width(face: &Face, text: &str, font_size_px: f32) -> f32 {
        let units_per_em = face.units_per_em() as f32;
        let mut width_units = 0.0;
        for ch in text.chars() {
            if let Some(glyph) = face.glyph_index(ch) {
                if let Some(advance) = face.glyph_hor_advance(glyph) {
                    width_units += advance as f32;
                }
            }
        }

        let font_size_pt = Unit::px_to_pt(font_size_px).0;

        let width_pt = width_units * font_size_pt / units_per_em;

        Unit::pt_to_px(width_pt)
    }

    pub fn measure_string(
        fonts: &PdfFonts,
        text: &str,
        font_size_px: f32,
        bold: bool,
        italic: bool,
    ) -> f32 {
        let pdf_font = if bold {
            &fonts.bold
        } else if italic {
            &fonts.italic
        } else {
            &fonts.regular
        };

        let face = match Face::parse(pdf_font.bytes, 0) {
            Ok(face) => face,
            Err(_) => return 0.0,
        };

        Self::measure_width(&face, text, font_size_px)
    }

    /// Đo chiều rộng của TextElement (giữ tương thích code cũ)
    pub fn measure_text(fonts: &PdfFonts, item: &TextElement) -> f32 {
        let font_weight = item.style.as_ref().and_then(|s| s.font_weight.as_deref());

        let bold = matches!(font_weight, Some(w) if w.eq_ignore_ascii_case("bold"));

        // Sau này nếu có fontStyle thì sửa tại đây
        let italic = false;

        Self::measure_string(fonts, &item.content, fonts.font_size(item), bold, italic)
    }

    pub fn calc_x(item: &TextElement, line_width: f32, align: &str) -> f32 {
        match align {
            "center" => item.x + (item.width - line_width) / 2.0,

            "right" => item.x + item.width - line_width,

            _ => item.x,
        }
    }

    pub fn calc_y(
        fonts: &PdfFonts,
        page_height: f32,
        item: &TextElement,
        layout: &TextLayoutResult,
    ) -> f32 {
        let font = fonts.font(item);
        let face = Face::parse(font.bytes, 0).unwrap();

        let units = face.units_per_em() as f32;
        let ascender = face.ascender() as f32;

        let baseline_pt = ascender * Unit::px_to_pt(fonts.font_size(item)).0 / units;

        let baseline_px = Unit::pt_to_px(baseline_pt);

        // let offset = if Self::is_center(item) {
        //     // Center cả chiều dọc
        //     (item.height - layout.height) / 2.0
        // } else {
        //     // Top
        //     0.0
        // };

        let offset = (item.height - layout.height) / 2.0;

        page_height - item.y - offset - baseline_px
    }

    pub fn wrap_text(fonts: &PdfFonts, item: &TextElement, data: &Value) -> TextLayoutResult {
        let font = fonts.font(item);
        let face = match Face::parse(font.bytes, 0) {
            Ok(face) => face,
            Err(_) => {
                let tokens = Tokenizer::tokenize(&item.content.clone().to_string());
                return TextLayoutResult {
                    lines: vec![TextLine {
                        runs: Parser::parse(&tokens, &data),
                        width: item.width,
                    }],
                    x: item.x,
                    y: item.y,
                    content_height: item.height,
                    content_width: item.width,
                    width: item.width,
                    height: fonts.font_size(item),
                    line_height: fonts.font_size(item) * 1.2,
                    base_y: 0.0,
                };
            }
        };

        let font_size = fonts.font_size(item);
        let max_width = item.width;

        let mut lines = Vec::<TextLine>::new();
        let mut current = String::new();

        for word in item.content.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", current, word)
            };

            let candidate_width = Self::measure_width(&face, &candidate, font_size);

            if candidate_width <= max_width {
                current = candidate;
            } else {
                if !current.is_empty() {
                    let width = Self::measure_width(&face, &current, font_size);
                    let tokens = Tokenizer::tokenize(&current);
                    lines.push(TextLine {
                        runs: Parser::parse(&tokens, &data),
                        width,
                    });
                }

                current = word.to_string();
            }
        }

        if !current.is_empty() {
            let width = Self::measure_width(&face, &current, font_size);
            let tokens = Tokenizer::tokenize(&current);
            lines.push(TextLine {
                runs: Parser::parse(&tokens, &data),
                width,
            });
        }

        let real_width = lines.iter().map(|l| l.width).fold(0.0_f32, f32::max);

        let line_height = font_size * 1.2;

        TextLayoutResult {
            height: lines.len() as f32 * line_height,
            x: item.x,
            y: item.y,
            content_height: item.height,
            content_width: item.width,
            width: real_width,
            line_height,
            lines,
            base_y: 0.0,
        }
    }

    fn is_center(item: &TextElement) -> bool {
        item.style
            .as_ref()
            .and_then(|s| s.text_align.as_deref())
            .map(|s| s.eq_ignore_ascii_case("center"))
            .unwrap_or(false)
    }

    pub fn build_context(data: &Value, source: &str, target: &str) -> Value {
        let mut map = serde_json::Map::new();

        if let Some(value) = resolve_value(data, source) {
            map.insert(target.to_string(), value);
        }

        Value::Object(map)
    }

    pub fn layout(
        fonts: &PdfFonts,
        page_height: f32,
        item: &TextElement,
        data: &Value,
    ) -> TextLayoutResult {
        let mut result = Self::wrap_text(fonts, item, data);
        result.base_y = Self::calc_y(fonts, page_height, item, &result);
        result
    }
}
