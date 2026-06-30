use crate::pdf::models::*;
use crate::pdf::text;
use crate::state::FONT;
use printpdf::{Mm, Op, ParsedFont, PdfDocument, PdfPage, PdfSaveOptions};

pub fn render(doc: PdfTemplate, output: &str) -> anyhow::Result<()> {
    let mut pdf = PdfDocument::new("Invoice");
    let mut ops = Vec::<Op>::new();
    let mut warnings = Vec::new();
    let parsed_font = ParsedFont::from_bytes(FONT, 0, &mut warnings)
        .ok_or_else(|| anyhow::anyhow!("Cannot parse font"))?;

    let font_id = pdf.add_font(&parsed_font);
    for e in doc.elements {
        match e {
            Element::Text(t) => {
                text::draw_text(&mut ops, font_id.clone(), doc.page.height, &t);
            }
        }
    }

    let page = PdfPage::new(Mm(doc.page.width), Mm(doc.page.height), ops);

    pdf.with_pages(vec![page]);

    let mut warnings = Vec::new();

    let bytes = pdf.save(&PdfSaveOptions::default(), &mut warnings);

    std::fs::write(output, bytes)?;

    Ok(())
}
