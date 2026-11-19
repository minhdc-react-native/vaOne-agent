use crate::pagination::constants::*;
use crate::pagination::page::PageLayout;
pub struct PaginationContext {
    pub page_height: f32,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub current_y: f32,
    pub original_bottom: f32,
    pub current_page: PageLayout,
    pub pages: Vec<PageLayout>,
    pub previous_design_bottom: f32,
}

impl PaginationContext {
    pub fn new(page_height: f32) -> Self {
        Self {
            page_height,
            margin_top: DEFAULT_MARGIN_TOP,
            margin_bottom: DEFAULT_MARGIN_BOTTOM,
            current_y: 0.0,
            original_bottom: 0.0,
            current_page: PageLayout::new(0),
            pages: Vec::new(),
            previous_design_bottom: 0.0,
        }
    }

    pub fn new_page(&mut self) {
        if !self.current_page.is_empty() {
            let next = self.pages.len() + 1;
            let page = std::mem::replace(&mut self.current_page, PageLayout::new(next));
            self.pages.push(page);
        } else {
            self.current_page.index = self.pages.len();
        }

        self.current_y = self.margin_top;
    }

    pub fn finish(mut self) -> Vec<PageLayout> {
        if !self.current_page.is_empty() {
            self.pages.push(self.current_page);
        }
        self.pages
    }
}
