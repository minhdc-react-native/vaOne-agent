use anyhow::Result;
use svgtypes::{PathParser, PathSegment};

use crate::models::{BoundingBox, Glyph, PathCommand, Point};

/// Parse SVG path thành PathCommand
pub fn parse_path(path: &str) -> Result<Vec<PathCommand>> {
    let mut commands = Vec::new();

    let mut current = Point::new(0.0, 0.0);
    let mut start = Point::new(0.0, 0.0);

    for seg in PathParser::from(path) {
        let seg = seg?;

        match seg {
            //------------------------------------------------
            // M
            //------------------------------------------------
            PathSegment::MoveTo { abs, x, y } => {
                let p = if abs {
                    Point::new(x, y)
                } else {
                    Point::new(current.x + x, current.y + y)
                };

                current = p;
                start = p;

                commands.push(PathCommand::MoveTo(p));
            }

            //------------------------------------------------
            // L
            //------------------------------------------------
            PathSegment::LineTo { abs, x, y } => {
                let p = if abs {
                    Point::new(x, y)
                } else {
                    Point::new(current.x + x, current.y + y)
                };

                current = p;

                commands.push(PathCommand::LineTo(p));
            }

            //------------------------------------------------
            // H -> LineTo
            //------------------------------------------------
            PathSegment::HorizontalLineTo { abs, x } => {
                let p = if abs {
                    Point::new(x, current.y)
                } else {
                    Point::new(current.x + x, current.y)
                };

                current = p;

                commands.push(PathCommand::LineTo(p));
            }

            //------------------------------------------------
            // V -> LineTo
            //------------------------------------------------
            PathSegment::VerticalLineTo { abs, y } => {
                let p = if abs {
                    Point::new(current.x, y)
                } else {
                    Point::new(current.x, current.y + y)
                };

                current = p;

                commands.push(PathCommand::LineTo(p));
            }

            //------------------------------------------------
            // Q
            //------------------------------------------------
            PathSegment::Quadratic { abs, x1, y1, x, y } => {
                let control = if abs {
                    Point::new(x1, y1)
                } else {
                    Point::new(current.x + x1, current.y + y1)
                };

                let end = if abs {
                    Point::new(x, y)
                } else {
                    Point::new(current.x + x, current.y + y)
                };

                current = end;

                commands.push(PathCommand::QuadTo { control, end });
            }

            //------------------------------------------------
            // C
            //------------------------------------------------
            PathSegment::CurveTo {
                abs,
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            } => {
                let c1 = if abs {
                    Point::new(x1, y1)
                } else {
                    Point::new(current.x + x1, current.y + y1)
                };

                let c2 = if abs {
                    Point::new(x2, y2)
                } else {
                    Point::new(current.x + x2, current.y + y2)
                };

                let end = if abs {
                    Point::new(x, y)
                } else {
                    Point::new(current.x + x, current.y + y)
                };

                current = end;

                commands.push(PathCommand::CubicTo {
                    control1: c1,
                    control2: c2,
                    end,
                });
            }

            //------------------------------------------------
            // Z
            //------------------------------------------------
            PathSegment::ClosePath { .. } => {
                current = start;
                commands.push(PathCommand::Close);
            }

            //------------------------------------------------
            // Các lệnh khác (Arc, Smooth...) hiện chưa dùng
            //------------------------------------------------
            _ => {}
        }
    }

    Ok(commands)
}

/// Tính bounding box
pub fn calculate_bbox(commands: &[PathCommand]) -> BoundingBox {
    let mut bbox = BoundingBox::default();

    for cmd in commands {
        match cmd {
            PathCommand::MoveTo(p) | PathCommand::LineTo(p) => {
                bbox.update(*p);
            }

            PathCommand::QuadTo { control, end } => {
                bbox.update(*control);
                bbox.update(*end);
            }

            PathCommand::CubicTo {
                control1,
                control2,
                end,
            } => {
                bbox.update(*control1);
                bbox.update(*control2);
                bbox.update(*end);
            }

            PathCommand::Horizontal(v) => {
                bbox.update(Point::new(*v, bbox.min_y));
            }

            PathCommand::Vertical(v) => {
                bbox.update(Point::new(bbox.min_x, *v));
            }

            PathCommand::Close => {}
        }
    }

    bbox
}

/// Normalize trực tiếp Glyph
pub fn normalize(glyph: &mut Glyph) -> Result<()> {
    let commands = parse_path(&glyph.raw_path)?;

    let bbox = calculate_bbox(&commands);

    let width = bbox.width().max(1.0);
    let height = bbox.height().max(1.0);

    let normalized = commands
        .into_iter()
        .map(|cmd| normalize_command(cmd, &bbox, width, height))
        .collect();

    glyph.commands = normalized;
    glyph.bbox = bbox;

    Ok(())
}

fn normalize_point(p: Point, bbox: &BoundingBox, width: f64, height: f64) -> Point {
    Point {
        x: (p.x - bbox.min_x) / width,
        y: (p.y - bbox.min_y) / height,
    }
}

fn normalize_command(cmd: PathCommand, bbox: &BoundingBox, width: f64, height: f64) -> PathCommand {
    match cmd {
        PathCommand::MoveTo(p) => PathCommand::MoveTo(normalize_point(p, bbox, width, height)),

        PathCommand::LineTo(p) => PathCommand::LineTo(normalize_point(p, bbox, width, height)),

        PathCommand::QuadTo { control, end } => PathCommand::QuadTo {
            control: normalize_point(control, bbox, width, height),
            end: normalize_point(end, bbox, width, height),
        },

        PathCommand::CubicTo {
            control1,
            control2,
            end,
        } => PathCommand::CubicTo {
            control1: normalize_point(control1, bbox, width, height),
            control2: normalize_point(control2, bbox, width, height),
            end: normalize_point(end, bbox, width, height),
        },

        PathCommand::Horizontal(v) => PathCommand::Horizontal(v),

        PathCommand::Vertical(v) => PathCommand::Vertical(v),

        PathCommand::Close => PathCommand::Close,
    }
}
