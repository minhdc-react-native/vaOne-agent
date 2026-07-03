use crate::pdf::layout::TextLayout;
use crate::pdf::models::*;
use crate::pdf::text;
use crate::pdf::utils::{load_fonts, Unit};
use printpdf::{Op, PdfDocument, PdfPage, PdfSaveOptions};
pub fn render(doc: PdfTemplate, data: serde_json::Value, output: &str) -> anyhow::Result<()> {
    let mut pdf = PdfDocument::new("Invoice");
    let mut ops = Vec::<Op>::new();

    let fonts = load_fonts(&mut pdf)?;

    for e in doc.elements {
        match e {
            Element::Text(t) => {
                let layout = TextLayout::layout(&fonts, doc.page.height, &t, &data);

                text::draw_text(&mut ops, &fonts, &t, &layout);
            }
            Element::Table(_t) => {
                let fake = TextElement {
                    x: 50.0,
                    y: 50.0,
                    width: 200.0,
                    height: 20.0,
                    content: "TABLE (TODO)".to_string(),
                    field_name: Some("storeInfo.address".to_string()),
                    style: None,
                };
                let layout = TextLayout::layout(&fonts, doc.page.height, &fake, &data);
                text::draw_text(&mut ops, &fonts, &fake, &layout);
            }
        }
    }

    let page = PdfPage::new(
        Unit::px_to_mm(doc.page.width),
        Unit::px_to_mm(doc.page.height),
        ops,
    );

    pdf.with_pages(vec![page]);

    let mut warnings = Vec::new();

    let bytes = pdf.save(&PdfSaveOptions::default(), &mut warnings);

    std::fs::write(output, bytes)?;

    Ok(())
}
