use super::utils::Unit;
use base64::{engine::general_purpose::STANDARD, Engine};
use printpdf::*;

use crate::pdf::models::LRCElement;

const DEFAULT_DPI: f32 = 96.0;

/// Decode Base64 -> RawImage
fn load_raw_image(base64: &str) -> Result<RawImage, Box<dyn std::error::Error>> {
    let encoded = base64
        .split_once(',')
        .map(|(_, value)| value)
        .unwrap_or(base64);

    let bytes = STANDARD.decode(encoded)?;

    let mut warnings = Vec::new();

    Ok(RawImage::decode_from_bytes(&bytes, &mut warnings)?)
}

/// Hàm dùng chung để vẽ ảnh
fn draw_image(
    doc: &mut PdfDocument,
    ops: &mut Vec<Op>,
    image: RawImage,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    page_height: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let image_id = doc.add_image(&image);

    let img_width_mm = Unit::px_to_mm(image.width as f32);
    let img_height_mm = Unit::px_to_mm(image.height as f32);

    let target_width_mm = Unit::px_to_mm(width);
    let target_height_mm = Unit::px_to_mm(height);

    let scale_x = target_width_mm / img_width_mm;
    let scale_y = target_height_mm / img_height_mm;

    // Hệ tọa độ PDF: gốc dưới bên trái
    let pdf_y = page_height - y - height;

    ops.push(Op::UseXobject {
        id: image_id,
        transform: XObjectTransform {
            translate_x: Some(Unit::px_to_mm(x).into_pt()),
            translate_y: Some(Unit::px_to_mm(pdf_y).into_pt()),
            rotate: None,
            scale_x: Some(scale_x),
            scale_y: Some(scale_y),
            dpi: Some(DEFAULT_DPI),
        },
    });

    Ok(())
}

/// Render background
pub fn render_background_image(
    doc: &mut PdfDocument,
    ops: &mut Vec<Op>,
    src: Option<String>,
    page_width: f32,
    page_height: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(src) = src else {
        return Ok(());
    };

    let src = src.trim();

    if src.is_empty() {
        return Ok(());
    }

    let image = match load_raw_image(src) {
        Ok(img) => img,
        Err(e) => {
            println!("Load background image error: {e}");
            return Ok(());
        }
    };

    draw_image(
        doc,
        ops,
        image,
        0.0,
        0.0,
        page_width,
        page_height,
        page_height,
    )
}

/// Render image element
pub fn render_image(
    doc: &mut PdfDocument,
    ops: &mut Vec<Op>,
    element: &LRCElement,
    page_height: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(src) = &element.content else {
        return Ok(());
    };

    let src = src.trim();

    if src.is_empty() {
        return Ok(());
    }

    let image = match load_raw_image(src) {
        Ok(img) => img,
        Err(e) => {
            println!("Load image error: {e}");
            return Ok(());
        }
    };

    draw_image(
        doc,
        ops,
        image,
        element.x,
        element.y,
        element.width,
        element.height,
        page_height,
    )
}
