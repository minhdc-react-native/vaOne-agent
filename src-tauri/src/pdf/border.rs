use crate::pdf::fonts::PdfFonts;
use crate::pdf::models::ElementStyle;
use crate::pdf::utils::Unit;
use printpdf::{
    Line, LineDashPattern, LinePoint, Mm, Op, PaintMode, Point, Polygon, PolygonRing, Pt,
    WindingOrder,
};
pub struct Border;

impl Border {
    pub fn draw_rect(
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
        let r = radius.min(width * 0.5).min(height * 0.5);

        let mut pts = Vec::<LinePoint>::new();

        if r <= f32::EPSILON {
            pts.push(lp(x, y));
            pts.push(lp(x + width, y));
            pts.push(lp(x + width, y - height));
            pts.push(lp(x, y - height));
        } else {
            const K: f32 = 0.552_284_75;

            // -------- Top --------
            pts.push(lp(x + r, y));
            pts.push(lp(x + width - r, y));

            // Top Right
            pts.extend([
                bp(x + width - r + K * r, y),
                bp(x + width, y - r + K * r),
                bp(x + width, y - r),
            ]);

            // Right
            pts.push(lp(x + width, y - height + r));

            // Bottom Right
            pts.extend([
                bp(x + width, y - height + r - K * r),
                bp(x + width - r + K * r, y - height),
                bp(x + width - r, y - height),
            ]);

            // Bottom
            pts.push(lp(x + r, y - height));

            // Bottom Left
            pts.extend([
                bp(x + r - K * r, y - height),
                bp(x, y - height + r - K * r),
                bp(x, y - height + r),
            ]);

            // Left
            pts.push(lp(x, y - r));

            // Top Left
            pts.extend([bp(x, y - r + K * r), bp(x + r - K * r, y), bp(x + r, y)]);
        }

        ops.push(Op::SaveGraphicsState);

        if let Some(bg) = background_color {
            if bg != "transparent" {
                ops.push(Op::SetFillColor {
                    col: fonts.parse_color(bg),
                });
            }
        }

        if let Some(border) = border_color {
            ops.push(Op::SetOutlineColor {
                col: fonts.parse_color(border),
            });
        }

        if let Some(width) = border_width {
            ops.push(Op::SetOutlineThickness { pt: Pt(width) });
        }

        Self::apply_border_style(ops, border_style);

        let has_background = background_color
            .map(|c| !c.eq_ignore_ascii_case("transparent"))
            .unwrap_or(false);

        let has_border = border_color.is_some() && border_width.unwrap_or(0.0) > 0.0;

        let mode = match (has_background, has_border) {
            (true, true) => PaintMode::FillStroke,
            (true, false) => PaintMode::Fill,
            (false, true) => PaintMode::Stroke,
            (false, false) => {
                ops.push(Op::RestoreGraphicsState);
                return;
            }
        };

        ops.push(Op::DrawPolygon {
            polygon: Polygon {
                rings: vec![PolygonRing { points: pts }],
                mode,
                winding_order: WindingOrder::NonZero,
            },
        });

        ops.push(Op::RestoreGraphicsState);
    }

    pub fn draw_line(
        ops: &mut Vec<Op>,
        fonts: &PdfFonts,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        border_color: Option<&str>,
        border_style: Option<&str>,
    ) {
        let line_width = (width / 2.0).min((height / 2.0)).max(0.1);

        // Độ dày nét
        ops.push(Op::SetOutlineThickness { pt: Pt(line_width) });

        // Màu nét
        if let Some(color) = border_color {
            ops.push(Op::SetOutlineColor {
                col: fonts.parse_color(color),
            });
        }

        // Kiểu nét
        Self::apply_border_style(ops, border_style);

        let (start, end) = if width >= height {
            // Đường ngang, nằm giữa chiều cao
            let cy = y + height / 2.0;

            (
                Point::new(Unit::px_to_mm(x), Unit::px_to_mm(cy)),
                Point::new(Unit::px_to_mm(x + width), Unit::px_to_mm(cy)),
            )
        } else {
            // Đường dọc, nằm giữa chiều rộng
            let cx = x + width / 2.0;

            (
                Point::new(Unit::px_to_mm(cx), Unit::px_to_mm(y - height)),
                Point::new(Unit::px_to_mm(cx), Unit::px_to_mm(y)),
            )
        };

        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: start,
                        bezier: false,
                    },
                    LinePoint {
                        p: end,
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });

        // Khôi phục về nét liền để không ảnh hưởng các hình sau
        ops.push(Op::SetLineDashPattern {
            dash: LineDashPattern::default(),
        });
    }

    fn apply_border_style(ops: &mut Vec<Op>, border_style: Option<&str>) {
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
    }
}

fn lp(x: f32, y: f32) -> LinePoint {
    LinePoint {
        p: Point::new(Unit::px_to_mm(x), Unit::px_to_mm(y)),
        bezier: false,
    }
}

fn bp(x: f32, y: f32) -> LinePoint {
    LinePoint {
        p: Point::new(Unit::px_to_mm(x), Unit::px_to_mm(y)),
        bezier: true,
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
    pts.extend([bp(x1, y1), bp(x2, y2), bp(x3, y3)]);
}
