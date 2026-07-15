use crate::models::TextElement;
use crate::text::FONT_SIZE;
use crate::utils::hex_to_color;
use printpdf::*;

#[derive(Clone)]
pub struct PdfFont {
    pub id: FontId,
    pub parsed: ParsedFont,
    pub bytes: &'static [u8],
}

#[derive(Clone)]
pub struct PdfFonts {
    pub regular: PdfFont,
    pub bold: PdfFont,
    pub italic: PdfFont,
}

impl PdfFonts {
    /// Chọn font theo style
    pub fn font(&self, item: &TextElement) -> &PdfFont {
        match item.style.as_ref().and_then(|s| s.font_weight.as_deref()) {
            Some(weight) if weight.eq_ignore_ascii_case("bold") => &self.bold,

            // Sau này:
            // Some(weight) if weight.eq_ignore_ascii_case("italic") => &self.italic,
            _ => &self.regular,
        }
    }

    /// Font size
    pub fn font_size(&self, item: &TextElement) -> f32 {
        item.style
            .as_ref()
            .and_then(|s| s.font_size)
            .unwrap_or(FONT_SIZE)
    }

    /// left / center / right
    pub fn text_align<'a>(&self, item: &'a TextElement) -> &'a str {
        item.style
            .as_ref()
            .and_then(|s| s.text_align.as_deref())
            .unwrap_or("left")
    }

    /// Màu chữ
    pub fn color(&self, item: &TextElement) -> Color {
        let hex = item
            .style
            .as_ref()
            .and_then(|s| s.color.as_deref())
            .unwrap_or("#000000");

        hex_to_color(hex)
    }

    /// Ước lượng chiều rộng text (tạm thời)
    ///
    /// Sau này sẽ thay bằng glyph metrics từ ParsedFont.
    pub fn measure_text(&self, item: &TextElement) -> f32 {
        let font_size = self.font_size(item);

        item.content.chars().count() as f32 * font_size * 0.52
    }

    pub fn font_by_style(&self, bold: bool, italic: bool) -> &PdfFont {
        match (bold, italic) {
            (true, false) => &self.bold,
            (false, true) => &self.italic,

            // Sau này nếu có font BoldItalic thì đổi ở đây
            (true, true) => &self.bold,

            _ => &self.regular,
        }
    }

    pub fn parse_color(&self, value: &str) -> Color {
        let hex = value.trim_start_matches('#');

        if hex.len() != 6 {
            return self.default_color();
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

        Color::Rgb(Rgb::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            None,
        ))
    }

    pub fn default_color(&self) -> Color {
        self.parse_color("#000000")
    }
}
