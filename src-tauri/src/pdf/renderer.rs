use crate::pdf::models::*;
use crate::pdf::text;
use crate::pdf::utils::load_fonts;
use printpdf::{Mm, Op, PdfDocument, PdfPage, PdfSaveOptions};

pub fn render(doc: PdfTemplate, output: &str) -> anyhow::Result<()> {
    let mut pdf = PdfDocument::new("Invoice");
    let mut ops = Vec::<Op>::new();

    let fonts = load_fonts(&mut pdf)?;

    for e in doc.elements {
        match e {
            Element::Text(t) => {
                text::draw_text(&mut ops, &fonts, doc.page.height, &t);
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

                text::draw_text(&mut ops, &fonts, doc.page.height, &fake);
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
