use crate::pdf::fonts::PdfFonts;
use crate::pdf::layout::TextLayout;
use crate::pdf::models::TextElement;
use crate::pdf::rich_text::RichTextParser;
use printpdf::*;

pub fn draw_text(ops: &mut Vec<Op>, fonts: &PdfFonts, page_height: f32, item: &TextElement) {
    let font = fonts.font(item);
    let font_size = fonts.font_size(item);
    let color = fonts.color(item);
    let align = fonts.text_align(item);

    // Tính toàn bộ layout
    let layout = TextLayout::layout(fonts, page_height, item);

    ops.push(Op::StartTextSection);

    ops.push(Op::SetFillColor { col: color });

    ops.push(Op::SetFontSize {
        font: font.id.clone(),
        size: Pt(font_size),
    });

    for (index, line) in layout.lines.iter().enumerate() {
        let x = TextLayout::calc_x(item, line.width, align);

        let y = layout.base_y - index as f32 * layout.line_height;
        let runs = RichTextParser::parse(&line.text);

        ops.push(Op::SetTextCursor {
            pos: Point {
                x: Mm(x).into(),
                y: Mm(y).into(),
            },
        });

        for run in runs {
            let font = if run.style.bold() {
                fonts.bold.id.clone()
            } else if run.style.italic() {
                fonts.italic.id.clone()
            } else {
                fonts.regular.id.clone()
            };

            ops.push(Op::SetFontSize {
                font: font.clone(),
                size: Pt(font_size),
            });

            ops.push(Op::WriteText {
                items: vec![TextItem::Text(run.text.clone())],
                font,
            });
        }
    }

    ops.push(Op::EndTextSection);
}
