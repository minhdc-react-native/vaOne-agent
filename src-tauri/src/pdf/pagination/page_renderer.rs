use super::page::PageLayout;
use super::paginator::PageItem;
use crate::pdf::{
    fonts::PdfFonts,
    image::render_image,
    table::table_render::TableRenderer,
    text,
    utils::Unit,
    utils::{draw_circle, draw_element_border, draw_line},
};

use printpdf::{Op, PdfDocument, PdfPage};

pub struct PageRenderer;

impl PageRenderer {
    pub fn render(
        doc: &mut PdfDocument,
        fonts: &PdfFonts,
        pages: Vec<PageLayout>,
        page_width: f32,
        page_height: f32,
        progress: impl Fn(usize, usize),
    ) -> anyhow::Result<()> {
        // let mut pdf_pages = Vec::new();
        let total = pages.len();
        for (index, page) in pages.into_iter().enumerate() {
            let mut ops = Vec::<Op>::new();

            for item in page.items {
                match item {
                    PageItem::Text { element, layout } => {
                        text::draw_text(&mut ops, fonts, &element, &layout, page_height);
                    }

                    PageItem::Table { element, layout } => {
                        if let Some(style) = &element.style {
                            TableRenderer::draw(&mut ops, &layout, fonts, page_height, style);
                        }
                    }

                    PageItem::Line { element, layout } => {
                        if let Some(style) = &element.style {
                            draw_line(
                                &mut ops,
                                fonts,
                                layout.x,
                                page_height - layout.y,
                                layout.width,
                                layout.height,
                                style,
                            );
                        }
                    }

                    PageItem::Rect { element, layout } => {
                        if let Some(style) = &element.style {
                            draw_element_border(
                                &mut ops,
                                fonts,
                                layout.x,
                                page_height - layout.y,
                                layout.width,
                                layout.height,
                                style,
                            );
                        }
                    }

                    PageItem::Circle { element, layout } => {
                        if let Some(style) = &element.style {
                            draw_circle(
                                &mut ops,
                                fonts,
                                layout.x,
                                page_height - layout.y,
                                layout.width,
                                layout.height,
                                style,
                            );
                        }
                    }

                    PageItem::Image { element, layout } => {
                        render_image(doc, &mut ops, &layout, page_height);
                    }

                    PageItem::Grid { element, layout } => {
                        // TODO
                    }
                }
            }
            // pdf_pages.push(PdfPage::new(
            //     Unit::px_to_mm(page_width),
            //     Unit::px_to_mm(page_height),
            //     ops,
            // ));
            doc.pages.push(PdfPage::new(
                Unit::px_to_mm(page_width),
                Unit::px_to_mm(page_height),
                ops,
            ));

            progress(index + 1, total);
        }

        // Ok(pdf_pages)
        Ok(())
    }
}
