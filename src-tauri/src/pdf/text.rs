use printpdf::*;

use crate::pdf::models::TextElement;
pub fn draw_text(ops: &mut Vec<Op>, font: FontId, page_height: f32, item: &TextElement) {
    let pdf_y = page_height - item.y;

    ops.push(Op::StartTextSection);

    ops.push(Op::SetFontSize {
        font: font.clone(),
        size: Pt(item.font_size),
    });

    ops.push(Op::SetTextCursor {
        pos: Point {
            x: Mm(item.x).into(),
            y: Mm(pdf_y).into(),
        },
    });

    ops.push(Op::WriteText {
        items: vec![TextItem::Text(item.value.clone())],
        font: font.clone(),
    });

    ops.push(Op::EndTextSection);
}
