use super::page::PageLayout;
use crate::pdf::{
    models::*,
    pagination::pagination_context::PaginationContext,
    table::models::{TableElement, TableLayoutResult},
};
#[derive(Debug, Clone)]
pub enum PageItem {
    Text {
        element: TextElement,
        layout: TextLayoutResult,
    },

    Table {
        element: TableElement,
        layout: TableLayoutResult,
    },

    Line {
        element: LRCElement,
        layout: LRCElement,
    },

    Rect {
        element: LRCElement,
        layout: LRCElement,
    },

    Circle {
        element: LRCElement,
        layout: LRCElement,
    },

    Image {
        element: LRCElement,
        layout: LRCElement,
    },

    Grid {
        element: GridElement,
        layout: GridElement,
    },
}

impl PageItem {
    pub fn name(&self) -> Option<String> {
        match self {
            Self::Text { element, .. } => element.clone().name,
            Self::Table { element, .. } => element.clone().name,
            Self::Line { element, .. } => element.clone().name,
            Self::Rect { element, .. } => element.clone().name,
            Self::Circle { element, .. } => element.clone().name,
            Self::Image { element, .. } => element.clone().name,
            Self::Grid { element, .. } => element.clone().name,
        }
    }
    pub fn x(&self) -> f32 {
        match self {
            Self::Text { element, layout } => layout.x,
            Self::Table { element, layout } => layout.x,
            Self::Line { element, layout } => layout.x,
            Self::Rect { element, layout } => layout.x,
            Self::Circle { element, layout } => layout.x,
            Self::Image { element, layout } => layout.x,
            Self::Grid { element, layout } => layout.x,
        }
    }
    pub fn design_x(&self) -> f32 {
        match self {
            Self::Text { element, .. } => element.x,
            Self::Table { element, .. } => element.x,
            Self::Line { element, .. } => element.x,
            Self::Rect { element, .. } => element.x,
            Self::Circle { element, .. } => element.x,
            Self::Image { element, .. } => element.x,
            Self::Grid { element, .. } => element.x,
        }
    }

    pub fn y(&self) -> f32 {
        match self {
            Self::Text { element, layout } => layout.y,
            Self::Table { element, layout } => layout.y,
            Self::Line { element, layout } => layout.y,
            Self::Rect { element, layout } => layout.y,
            Self::Circle { element, layout } => layout.y,
            Self::Image { element, layout } => layout.y,
            Self::Grid { element, layout } => layout.y,
        }
    }
    pub fn design_y(&self) -> f32 {
        match self {
            Self::Text { element, .. } => element.y,
            Self::Table { element, .. } => element.y,
            Self::Line { element, .. } => element.y,
            Self::Rect { element, .. } => element.y,
            Self::Circle { element, .. } => element.y,
            Self::Image { element, .. } => element.y,
            Self::Grid { element, .. } => element.y,
        }
    }

    pub fn height(&self) -> f32 {
        match self {
            Self::Text { element, layout } => layout.height,
            Self::Table { element, layout } => layout.height,
            Self::Line { element, layout } => layout.height,
            Self::Rect { element, layout } => layout.height,
            Self::Circle { element, layout } => layout.height,
            Self::Image { element, layout } => layout.height,
            Self::Grid { element, layout } => layout.height,
        }
    }
    pub fn design_height(&self) -> f32 {
        match self {
            Self::Text { element, .. } => element.height,
            Self::Table { element, .. } => element.height,
            Self::Line { element, .. } => element.height,
            Self::Rect { element, .. } => element.height,
            Self::Circle { element, .. } => element.height,
            Self::Image { element, .. } => element.height,
            Self::Grid { element, .. } => element.height,
        }
    }

    pub fn set_y(&mut self, y: f32) {
        match self {
            Self::Text { element, layout } => layout.y = y,
            Self::Table { element, layout } => layout.y = y,
            Self::Line { element, layout } => layout.y = y,
            Self::Rect { element, layout } => layout.y = y,
            Self::Circle { element, layout } => layout.y = y,
            Self::Image { element, layout } => layout.y = y,
            Self::Grid { element, layout } => layout.y = y,
        }
    }

    pub fn translate_y(&mut self, dy: f32) {
        match self {
            Self::Text { element, layout } => {
                layout.translate_y(dy);
            }

            Self::Table { element, layout } => {
                layout.translate_y(dy);
            }

            Self::Line { element, layout }
            | Self::Rect { element, layout }
            | Self::Circle { element, layout }
            | Self::Image { element, layout } => {
                layout.translate_y(dy);
            }

            Self::Grid { element, layout } => {
                layout.translate_y(dy);
            }
        }
    }

    pub fn bottom(&self) -> f32 {
        self.y() + self.height()
    }
    pub fn design_bottom(&self) -> f32 {
        self.design_y() + self.design_height()
    }
}
pub struct Paginator;

impl Paginator {
    pub fn paginate(
        items: Vec<PageItem>,
        _page_width: f32,
        page_height: f32,
        continuous: bool,
    ) -> anyhow::Result<(Vec<PageLayout>, f32)> {
        let mut ctx = PaginationContext::new(page_height);

        for item in items {
            match item {
                PageItem::Table { element, layout } => {
                    Self::paginate_table(element, layout, &mut ctx, continuous);
                }

                mut item => {
                    let spacing = item.design_y() - ctx.previous_design_bottom;
                    ctx.current_y += spacing;

                    let next_height = item.height();
                    let available_height = ctx.page_height - ctx.margin_bottom;

                    if !continuous
                        && ctx.current_y + next_height > available_height
                        && !ctx.current_page.is_empty()
                    {
                        ctx.new_page();
                    }

                    let diff = ctx.current_y - item.y();

                    item.translate_y(diff);
                    // if let Some(name) = item.name().as_deref() {
                    //     if name == "text_o3ak" || name == "****" {
                    //         println!(
                    //             "design_y={} layout_y={} current_y={} diff={}",
                    //             item.design_y(),
                    //             item.y(),
                    //             ctx.current_y,
                    //             ctx.current_y - item.y(),
                    //         );
                    //     }
                    // }
                    ctx.current_y = item.bottom();
                    ctx.previous_design_bottom = item.design_bottom();

                    ctx.current_page.push(item);
                }
            }
        }
        let ctx_margin_bottom = ctx.margin_bottom;
        let pages = ctx.finish();
        let height = if continuous {
            let mut max_bottom = 0.0;

            for page in &pages {
                for item in &page.items {
                    let bottom = match item {
                        PageItem::Table { layout, .. } => layout.bottom(),
                        _ => item.bottom(),
                    };

                    if bottom > max_bottom {
                        max_bottom = bottom;
                    }
                }
            }

            max_bottom + ctx_margin_bottom
        } else {
            page_height
        };
        Ok((pages, height))
    }

    fn paginate_table(
        element: TableElement,
        layout: TableLayoutResult,
        ctx: &mut PaginationContext,
        continuous: bool,
    ) {
        let design_bottom = element.y + element.height;
        let header_height = layout.header_height();

        // Một row còn lớn hơn cả trang thì hiện tại chưa hỗ trợ split
        if layout
            .rows
            .iter()
            .any(|r| r.height > ctx.page_height - header_height)
        {
            panic!("Table row is higher than available page height.");
        }

        let template = layout.empty_body();
        let rows = layout.rows;
        let spacing = element.y - ctx.previous_design_bottom;
        ctx.current_y += spacing;

        let mut table = template.clone();
        table.translate_y(ctx.current_y - table.y);

        for row in rows {
            // Chiều cao nếu thêm row này
            let next_height = table.height + row.height;

            let available_height = ctx.page_height - ctx.margin_bottom;
            // Không còn đủ chỗ trên trang hiện tại
            if !continuous
                && ctx.current_y + next_height > available_height
                && !table.rows.is_empty()
            {
                table.recalc_height();

                let mut table_element = element.clone();
                table_element.y = table.y;
                table_element.height = table.height;

                ctx.current_page.push(PageItem::Table {
                    element: table_element,
                    layout: table,
                });

                // Sang trang mới
                ctx.new_page();

                table = template.clone();
                table.translate_y(ctx.current_y - table.y);
            }

            table.push_row(row);
        }

        if !table.rows.is_empty() {
            table.recalc_height();

            let mut table_element = element;
            table_element.y = table.y;
            table_element.height = table.height;

            ctx.current_y = table.bottom();

            ctx.previous_design_bottom = design_bottom;

            ctx.current_page.push(PageItem::Table {
                element: table_element,
                layout: table,
            });
        }
    }
}
