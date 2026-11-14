use crate::{
    fingerprint::fingerprint,
    models::{Glyph, GlyphFeature, PathCommand},
};

/// Sinh vector đặc trưng cho một glyph
pub fn extract_feature(glyph: &Glyph, _character: Option<char>) -> GlyphFeature {
    let mut move_count = 0f32;
    let mut line_count = 0f32;
    let mut quad_count = 0f32;
    let mut cubic_count = 0f32;
    let mut close_count = 0f32;

    let mut path_length = 0f32;

    let mut last_x = 0f32;
    let mut last_y = 0f32;

    for cmd in &glyph.commands {
        match cmd {
            PathCommand::MoveTo(p) => {
                move_count += 1.0;
                last_x = p.x as f32;
                last_y = p.y as f32;
            }

            PathCommand::LineTo(p) => {
                line_count += 1.0;

                path_length += distance(last_x, last_y, p.x as f32, p.y as f32);

                last_x = p.x as f32;
                last_y = p.y as f32;
            }

            PathCommand::Horizontal(x) => {
                line_count += 1.0;

                path_length += (last_x - *x as f32).abs();

                last_x = *x as f32;
            }

            PathCommand::Vertical(y) => {
                line_count += 1.0;

                path_length += (last_y - *y as f32).abs();

                last_y = *y as f32;
            }

            PathCommand::QuadTo { control, end } => {
                quad_count += 1.0;

                path_length += distance(last_x, last_y, control.x as f32, control.y as f32);

                path_length += distance(
                    control.x as f32,
                    control.y as f32,
                    end.x as f32,
                    end.y as f32,
                );

                last_x = end.x as f32;
                last_y = end.y as f32;
            }

            PathCommand::CubicTo {
                control1,
                control2,
                end,
            } => {
                cubic_count += 1.0;

                path_length += distance(last_x, last_y, control1.x as f32, control1.y as f32);

                path_length += distance(
                    control1.x as f32,
                    control1.y as f32,
                    control2.x as f32,
                    control2.y as f32,
                );

                path_length += distance(
                    control2.x as f32,
                    control2.y as f32,
                    end.x as f32,
                    end.y as f32,
                );

                last_x = end.x as f32;
                last_y = end.y as f32;
            }

            PathCommand::Close => {
                close_count += 1.0;
            }
        }
    }

    let width = glyph.bbox.width() as f32;
    let height = glyph.bbox.height() as f32;

    let aspect_ratio = if height > 0.0 { width / height } else { 0.0 };

    //
    // Vector đặc trưng
    //
    let vector = vec![
        move_count,
        line_count,
        quad_count,
        cubic_count,
        close_count,
        width,
        height,
        aspect_ratio,
        path_length,
    ];

    GlyphFeature {
        hash: fingerprint(glyph),
        vector,
    }
}

#[inline]
fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}
