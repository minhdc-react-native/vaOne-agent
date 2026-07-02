use crate::pdf::models::TextElement;
use crate::pdf::utils::hex_to_color;
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
            .unwrap_or(12.0)
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
}
