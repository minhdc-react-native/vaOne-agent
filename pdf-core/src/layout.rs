use crate::fonts::PdfFonts;
use crate::models::{TextElement, TextLayoutResult, TextLine, TextRun};
use crate::template::{
    evaluator::Evaluator, models::FormatterContext, parser::Parser, tokenizer::Tokenizer,
};
use crate::text::LINE_HEIGHT;
use crate::utils::{resolve_value, Unit};
use serde_json::{Map, Value};
use ttf_parser::Face;

pub struct TextLayout;

impl TextLayout {
    fn measure_width(face: &Face, text: &str, font_size_px: f32) -> f32 {
        // lines.push(TextLine {
        //     runs: Parser::parse(&tokens, &data),
        //     width,
        // });

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

        let offset = if Self::is_center(item) {
            // Center cả chiều dọc
            (item.height - layout.height) / 2.0
        } else {
            // Top
            0.0
        };

        // let offset = (item.height - layout.height) / 2.0;

        page_height - layout.y - offset - baseline_px
    }

    pub fn wrap_text(
        fonts: &PdfFonts,
        item: &TextElement,
        data: &Value,
        ctx: FormatterContext,
    ) -> TextLayoutResult {
        let font_size = fonts.font_size(item);
        let max_width = item.width;

        let tokens = Tokenizer::tokenize(&item.content);

        let runs = Parser::parse(&tokens, data, ctx);

        let words = Self::split_runs(runs);

        let mut lines = Vec::<TextLine>::new();

        let mut current_runs = Vec::<TextRun>::new();

        let mut current_width = 0.0;

        for run in words {
            let width = Self::measure_run_width(fonts, &run, font_size);

            // Từ bình thường
            if width <= max_width {
                if current_width + width <= max_width {
                    current_width += width;
                    current_runs.push(run);
                } else {
                    if !current_runs.is_empty() {
                        lines.push(TextLine {
                            runs: current_runs,
                            width: current_width,
                        });
                    }

                    current_runs = vec![run];
                    current_width = width;
                }

                continue;
            }

            // -------------------------
            // Từ quá dài -> cắt theo ký tự
            // -------------------------
            for ch_run in Self::split_run_by_char(run) {
                let ch_width = Self::measure_run_width(fonts, &ch_run, font_size);

                if current_width + ch_width <= max_width {
                    current_width += ch_width;
                    current_runs.push(ch_run);
                } else {
                    if !current_runs.is_empty() {
                        lines.push(TextLine {
                            runs: current_runs,
                            width: current_width,
                        });
                    }

                    current_runs = vec![ch_run];
                    current_width = ch_width;
                }
            }
        }

        if !current_runs.is_empty() {
            lines.push(TextLine {
                runs: current_runs,
                width: current_width,
            });
        }

        let real_width = lines.iter().map(|l| l.width).fold(0.0_f32, f32::max);

        let line_height = font_size * LINE_HEIGHT;
        let auto_height = item.auto_height.unwrap_or(false);

        let height = if auto_height {
            lines.len() as f32 * line_height
        } else {
            item.height
        };

        TextLayoutResult {
            height,
            x: item.x,
            y: item.y,
            content_height: item.height,
            content_width: item.width,
            width: real_width,
            line_height,
            lines,
            base_y: 0.0,
            visible: None,
        }
    }

    fn measure_run_width(fonts: &PdfFonts, run: &TextRun, default_font_size: f32) -> f32 {
        let font = fonts.font_by_style(run.style.bold, run.style.italic);

        let face = Face::parse(font.bytes, 0).unwrap();

        let size = run.style.font_size.unwrap_or(default_font_size);

        Self::measure_width(&face, &run.text, size)
    }

    fn split_runs(runs: Vec<TextRun>) -> Vec<TextRun> {
        let mut out = Vec::new();

        for run in runs {
            let mut current = String::new();

            for ch in run.text.chars() {
                if ch.is_whitespace() {
                    if !current.is_empty() {
                        let mut r = run.clone();
                        r.text = std::mem::take(&mut current);
                        out.push(r);
                    }

                    let mut r = run.clone();
                    r.text = ch.to_string();
                    out.push(r);
                } else {
                    current.push(ch);
                }
            }

            if !current.is_empty() {
                let mut r = run.clone();
                r.text = current;
                out.push(r);
            }
        }

        out
    }

    fn split_run_by_char(run: TextRun) -> Vec<TextRun> {
        run.text
            .chars()
            .map(|ch| {
                let mut r = run.clone();
                r.text = ch.to_string();
                r
            })
            .collect()
    }

    fn is_center(item: &TextElement) -> bool {
        item.style
            .as_ref()
            .and_then(|style| style.center_y)
            .unwrap_or(false)
    }

    pub fn build_context(data: &Value, source: &str, target: &str, watch: &[String]) -> Value {
        let mut map = Map::new();

        if let Some(value) = resolve_value(data, source) {
            Self::insert_nested(&mut map, source, value.clone());
            Self::insert_nested(&mut map, target, value);
        }

        for field in watch {
            if let Some(value) = resolve_value(data, field) {
                Self::insert_nested(&mut map, field, value);
            }
        }

        Value::Object(map)
    }

    fn insert_nested(map: &mut Map<String, Value>, path: &str, value: Value) {
        let parts: Vec<&str> = path.split('.').collect();

        fn helper(current: &mut Map<String, Value>, parts: &[&str], value: Value) {
            if parts.len() == 1 {
                current.insert(parts[0].to_string(), value);
                return;
            }

            let entry = current
                .entry(parts[0].to_string())
                .or_insert_with(|| Value::Object(Map::new()));

            match entry {
                Value::Object(obj) => helper(obj, &parts[1..], value),
                _ => {
                    // Nếu key đã tồn tại nhưng không phải Object thì ghi đè
                    let mut obj = Map::new();
                    helper(&mut obj, &parts[1..], value);
                    *entry = Value::Object(obj);
                }
            }
        }

        helper(map, &parts, value);
    }

    pub fn layout(
        fonts: &PdfFonts,
        page_height: f32,
        item: &TextElement,
        data: &Value,
        ctx: FormatterContext,
    ) -> TextLayoutResult {
        let mut result = Self::wrap_text(fonts, item, data, ctx);
        result.base_y = Self::calc_y(fonts, page_height, item, &result);
        result
    }
}
