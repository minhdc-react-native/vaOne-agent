use anyhow::{Result, bail};
use roxmltree::Document;

use crate::{
    models::Glyph,
    normalize::{calculate_bbox, parse_path},
};

/// Parse toàn bộ SVG thành danh sách glyph.
///
/// Chỉ lấy các path có thuộc tính `fill`.
/// Các path chỉ có `stroke` sẽ bị bỏ qua.
pub fn parse_svg(svg: &str) -> Result<Vec<Glyph>> {
    let doc = Document::parse(svg)?;

    let mut glyphs = Vec::new();

    for node in doc.descendants().filter(|n| n.has_tag_name("path")) {
        // Bỏ đường nhiễu
        if let Some(fill) = node.attribute("fill") {
            if fill.eq_ignore_ascii_case("none") {
                continue;
            }
        }

        let raw_path = match node.attribute("d") {
            Some(v) if !v.trim().is_empty() => v.to_string(),
            _ => continue,
        };

        let commands = parse_path(&raw_path)?;

        if commands.is_empty() {
            continue;
        }

        let bbox = calculate_bbox(&commands);

        glyphs.push(Glyph {
            raw_path,
            commands,
            bbox,
            fingerprint: None,
        });
    }

    if glyphs.is_empty() {
        bail!("No glyph found in SVG");
    }

    // Sắp xếp từ trái sang phải
    glyphs.sort_by(|a, b| {
        a.bbox
            .min_x
            .partial_cmp(&b.bbox.min_x)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(glyphs)
}

/// Đếm số glyph trong SVG
pub fn glyph_count(svg: &str) -> Result<usize> {
    Ok(parse_svg(svg)?.len())
}

/// Lấy path gốc
pub fn raw_paths(svg: &str) -> Result<Vec<String>> {
    Ok(parse_svg(svg)?.into_iter().map(|g| g.raw_path).collect())
}
