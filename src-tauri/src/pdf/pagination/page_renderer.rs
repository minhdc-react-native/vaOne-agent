use super::page::PageLayout;
use super::paginator::PageItem;
use crate::pdf::{
    fonts::PdfFonts,
    image::render_image,
    table::table_render::TableRenderer,
    text,
    utils::{draw_circle, draw_element_border, draw_line},
};

use printpdf::{Op, PdfDocument, PdfPage};

pub struct PageRenderer;

impl PageRenderer {
    pub fn render(
        pdf: &mut PdfDocument,
        fonts: &PdfFonts,
        pages: Vec<PageLayout>,
        page_width: f32,
        page_height: f32,
    ) -> anyhow::Result<Vec<PdfPage>> {
        let mut pdf_pages = Vec::new();

        for page in pages {
            let mut ops = Vec::<Op>::new();

            for item in page.items {
                match item {
                    PageItem::Text { element, layout } => {
                        text::draw_text(&mut ops, fonts, &element, &layout);
                    }

                    PageItem::Table { element, layout } => {
                        if let Some(style) = &element.style {
                            TableRenderer::draw(&mut ops, &layout, fonts, page_height, style);
                        }
                    }

                    PageItem::Line { element } => {
                        if let Some(style) = &element.style {
                            draw_line(
                                &mut ops,
                                fonts,
                                element.x,
                                page_height - element.y,
                                element.width,
                                element.height,
                                style,
                            );
                        }
                    }

                    PageItem::Rect { element } => {
                        if let Some(style) = &element.style {
                            draw_element_border(
                                &mut ops,
                                fonts,
                                element.x,
                                page_height - element.y,
                                element.width,
                                element.height,
                                style,
                            );
                        }
                    }

                    PageItem::Circle { element } => {
                        if let Some(style) = &element.style {
                            draw_circle(
                                &mut ops,
                                fonts,
                                element.x,
                                page_height - element.y,
                                element.width,
                                element.height,
                                style,
                            );
                        }
                    }

                    PageItem::Image { element } => {
                        render_image(pdf, &mut ops, &element, page_height);
                    }

                    PageItem::Grid { element } => {
                        // TODO
                    }
                }
            }
        }

        Ok(pdf_pages)
    }
}
