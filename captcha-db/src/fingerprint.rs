use sha2::{Digest, Sha256};

use crate::models::{Glyph, PathCommand, Point};

/// Sinh SHA256 fingerprint từ glyph
pub fn fingerprint(glyph: &Glyph) -> String {
    let text = serialize(glyph);

    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());

    hex::encode(hasher.finalize())
}

/// Serialize glyph thành chuỗi canonical
pub fn serialize(glyph: &Glyph) -> String {
    let mut out = String::new();

    for cmd in &glyph.commands {
        match cmd {
            PathCommand::MoveTo(p) => {
                out.push('M');
                write_point(&mut out, *p);
            }

            PathCommand::LineTo(p) => {
                out.push('L');
                write_point(&mut out, *p);
            }

            PathCommand::Horizontal(v) => {
                out.push('H');
                out.push_str(&format!("{:.4};", v));
            }

            PathCommand::Vertical(v) => {
                out.push('V');
                out.push_str(&format!("{:.4};", v));
            }

            PathCommand::QuadTo { control, end } => {
                out.push('Q');
                write_point(&mut out, *control);
                write_point(&mut out, *end);
            }

            PathCommand::CubicTo {
                control1,
                control2,
                end,
            } => {
                out.push('C');
                write_point(&mut out, *control1);
                write_point(&mut out, *control2);
                write_point(&mut out, *end);
            }

            PathCommand::Close => {
                out.push('Z');
            }
        }
    }

    out
}

fn write_point(out: &mut String, p: Point) {
    out.push_str(&format!("{:.4},{:.4};", p.x, p.y));
}

/// SHA256 rút gọn để debug
pub fn short_fingerprint(glyph: &Glyph) -> String {
    fingerprint(glyph)[0..16].to_string()
}

/// So sánh 2 glyph
pub fn is_same(a: &Glyph, b: &Glyph) -> bool {
    fingerprint(a) == fingerprint(b)
}

/// Dump canonical path
pub fn dump(glyph: &Glyph) {
    println!("{}", serialize(glyph));
}
