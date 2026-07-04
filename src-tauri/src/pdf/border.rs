use crate::pdf::fonts::PdfFonts;
use crate::pdf::utils::Unit;
use printpdf::{
    LineDashPattern, LinePoint, Mm, Op, PaintMode, Point, Polygon, PolygonRing, Pt, WindingOrder,
};
pub struct Border;

impl Border {
    pub fn draw(
        ops: &mut Vec<Op>,
        fonts: &PdfFonts,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        background_color: Option<&str>,
        border_color: Option<&str>,
        border_width: Option<f32>,
        border_style: Option<&str>,
    ) {
        let r = radius.min(width / 2.0).min(height / 2.0);

        let x: f32 = x;
        let y = y - 6.0;
        let w = width;
        let h = height;

        // Hệ số Bezier để xấp xỉ cung tròn
        const K: f32 = 0.55228475;

        let mut pts = Vec::<LinePoint>::new();

        // ---------- Top ----------
        pts.push(lp(x + r, y));
        pts.push(lp(x + w - r, y));

        // ---------- Top Right ----------
        bezier(
            &mut pts,
            x + w - r,
            y,
            x + w - r + K * r,
            y,
            x + w,
            y + r - K * r,
            x + w,
            y + r,
        );

        // ---------- Right ----------
        pts.push(lp(x + w, y + h - r));

        // ---------- Bottom Right ----------
        bezier(
            &mut pts,
            x + w,
            y + h - r,
            x + w,
            y + h - r + K * r,
            x + w - r + K * r,
            y + h,
            x + w - r,
            y + h,
        );

        // ---------- Bottom ----------
        pts.push(lp(x + r, y + h));

        // ---------- Bottom Left ----------
        bezier(
            &mut pts,
            x + r,
            y + h,
            x + r - K * r,
            y + h,
            x,
            y + h - r + K * r,
            x,
            y + h - r,
        );

        // ---------- Left ----------
        pts.push(lp(x, y + r));

        // ---------- Top Left ----------
        bezier(
            &mut pts,
            x,
            y + r,
            x,
            y + r - K * r,
            x + r - K * r,
            y,
            x + r,
            y,
        );

        ops.push(Op::SaveGraphicsState);

        if let Some(bg_color) = background_color {
            if bg_color != "transparent" {
                ops.push(Op::SetFillColor {
                    col: fonts.parse_color(bg_color),
                });
            }
        }

        if let Some(b_color) = border_color {
            ops.push(Op::SetOutlineColor {
                col: fonts.parse_color(b_color),
            });
        }

        if let Some(b_width) = border_width {
            ops.push(Op::SetOutlineThickness {
                pt: Pt(b_width / 2.0),
            });
        }

        if let Some(b_style) = border_style {
            if b_style == "dashed" {
                ops.push(Op::SetLineDashPattern {
                    dash: LineDashPattern {
                        offset: 0,
                        dash_1: Some(2),
                        gap_1: Some(1),
                        dash_2: None,
                        gap_2: None,
                        dash_3: None,
                        gap_3: None,
                    },
                });
            } else {
                // solid hoặc giá trị khác
                ops.push(Op::SetLineDashPattern {
                    dash: LineDashPattern::default(),
                });
            }
        }
        let has_background = background_color
            .map(|c| !c.eq_ignore_ascii_case("transparent"))
            .unwrap_or(false);

        let has_border = border_color.is_some() && border_width.unwrap_or(0.0) > 0.0;
        let mode = match (has_background, has_border) {
            (true, true) => PaintMode::FillStroke,
            (true, false) => PaintMode::Fill,
            (false, true) => PaintMode::Stroke,
            (false, false) => return,
        };

        let polygon = Polygon {
            rings: vec![PolygonRing { points: pts }],
            mode,
            winding_order: WindingOrder::NonZero,
        };
        ops.push(Op::DrawPolygon { polygon });

        ops.push(Op::RestoreGraphicsState);
    }
}

fn lp(x: f32, y: f32) -> LinePoint {
    LinePoint {
        p: Point::new(Unit::px_to_mm(x), Unit::px_to_mm(y)),
        bezier: false,
    }
}

fn bezier(
    pts: &mut Vec<LinePoint>,
    _x0: f32,
    _y0: f32,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    x3: f32,
    y3: f32,
) {
    pts.push(LinePoint {
        p: Point::new(Unit::px_to_mm(x1), Unit::px_to_mm(y1)),
        bezier: true,
    });

    pts.push(LinePoint {
        p: Point::new(Unit::px_to_mm(x2), Unit::px_to_mm(y2)),
        bezier: true,
    });

    pts.push(LinePoint {
        p: Point::new(Unit::px_to_mm(x3), Unit::px_to_mm(y3)),
        bezier: true,
    });
}
