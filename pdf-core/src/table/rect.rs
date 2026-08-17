use crate::{fonts::PdfFonts, utils::Unit};
use printpdf::{LinePoint, Op, PaintMode, Point, Polygon, PolygonRing, WindingOrder};

pub struct Rect;

impl Rect {
    pub fn fill(
        ops: &mut Vec<Op>,
        fonts: &PdfFonts,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: &str,
    ) {
        ops.push(Op::SetFillColor {
            col: fonts.parse_color(color),
        });

        let polygon = Polygon {
            rings: vec![PolygonRing {
                points: vec![
                    lp(x, y),
                    lp(x + width, y),
                    lp(x + width, y + height),
                    lp(x, y + height),
                ],
            }],
            mode: PaintMode::Fill,
            winding_order: WindingOrder::NonZero,
        };

        ops.push(Op::DrawPolygon { polygon });
    }
}

fn lp(x: f32, y: f32) -> LinePoint {
    LinePoint {
        p: Point::new(Unit::px_to_mm(x), Unit::px_to_mm(y)),
        bezier: false,
    }
}
