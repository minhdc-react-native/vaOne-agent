use crate::border::Border;
use crate::fonts::PdfFonts;
use crate::layout::TextLayout;
use crate::models::{ElementStyle, TextElement, TextLayoutResult, TextStyle};
use crate::utils::Unit;
use printpdf::{Op, Point, TextItem};
pub const FONT_SIZE: f32 = 12.0;
pub const LINE_HEIGHT: f32 = 1.2;

pub fn draw_text(
    ops: &mut Vec<Op>,
    fonts: &PdfFonts,
    item: &TextElement,
    layout: &TextLayoutResult,
    page_height: f32,
) {
    let base_style = item.style.clone().unwrap_or_default();
    let align = fonts.text_align(item);

    for (index, line) in layout.lines.iter().enumerate() {
        let mut width = 0.0;

        for run in &line.runs {
            let style = merge_style(&base_style, &run.style);
            width += TextLayout::measure_string(
                fonts,
                &run.text,
                style.font_size.unwrap_or(FONT_SIZE),
                style.bold,
                style.italic,
            );
        }

        let x = TextLayout::calc_x(item, width, align);
        let base_y = TextLayout::calc_y(fonts, page_height, item, layout);
        let y = base_y - index as f32 * layout.line_height;

        // if let Some(name) = item.name.as_deref() {
        //     if name == "text_o3ak" || name == "****" {
        //         println!(
        //             "design_y={} layout_y={} base_y={}",
        //             item.y, layout.y, base_y
        //         );
        //     }
        // }

        ops.push(Op::StartTextSection);

        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Unit::px_to_mm(x).into(),
                y: Unit::px_to_mm(y).into(),
            },
        });

        let mut current_x = x;
        for run in &line.runs {
            let style = merge_style(&base_style, &run.style);

            let font = fonts.font_by_style(style.bold, style.italic);

            let color = style
                .color
                .as_ref()
                .map(|c| fonts.parse_color(c))
                .unwrap_or_else(|| fonts.default_color());

            let font_size = style.font_size.unwrap_or(FONT_SIZE);

            let run_width =
                TextLayout::measure_string(fonts, &run.text, font_size, style.bold, style.italic);

            ops.push(Op::SetFillColor { col: color });

            ops.push(Op::SetFontSize {
                font: font.id.clone(),
                size: Unit::px_to_pt(font_size),
            });

            ops.push(Op::WriteText {
                items: vec![TextItem::Text(run.text.clone())],
                font: font.id.clone(),
            });

            // underline
            if style.underline {
                Border::draw_line(
                    ops,
                    fonts,
                    current_x,
                    y - font_size * 0.08, // dưới baseline
                    run_width,
                    1.0,
                    style.color.as_deref(),
                    Some("solid"),
                );
            }

            // line-through
            if style.strike {
                Border::draw_line(
                    ops,
                    fonts,
                    current_x,
                    y + font_size * 0.30, // giữa chữ
                    run_width,
                    1.0,
                    style.color.as_deref(),
                    Some("solid"),
                );
            }

            current_x += run_width;
        }

        ops.push(Op::EndTextSection);
    }
}

fn to_text_style(style: &ElementStyle) -> TextStyle {
    TextStyle {
        bold: matches!(style.font_weight.as_deref(), Some("bold")),
        italic: matches!(style.font_style.as_deref(), Some("italic")),
        underline: matches!(style.text_decoration.as_deref(), Some("underline")),
        strike: matches!(style.text_decoration.as_deref(), Some("line-through")),
        color: style.color.clone(),
        font_size: style.font_size,
    }
}
fn merge_style(base: &ElementStyle, inline: &TextStyle) -> TextStyle {
    let base_style = to_text_style(base);

    TextStyle {
        bold: inline.bold || base_style.bold,
        italic: inline.italic || base_style.italic,
        underline: inline.underline || base_style.underline,
        strike: inline.strike || base_style.strike,
        color: inline.color.clone().or(base_style.color),
        font_size: inline.font_size.or(base_style.font_size),
    }
}
