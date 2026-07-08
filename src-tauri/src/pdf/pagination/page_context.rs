use super::constants::*;

#[derive(Debug, Clone)]
pub struct PageContext {
    pub width: f32,
    pub height: f32,

    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
}

impl PageContext {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,

            margin_top: DEFAULT_MARGIN_TOP,
            margin_right: DEFAULT_MARGIN_RIGHT,
            margin_bottom: DEFAULT_MARGIN_BOTTOM,
            margin_left: DEFAULT_MARGIN_LEFT,
        }
    }

    pub fn content_width(&self) -> f32 {
        self.width - self.margin_left - self.margin_right
    }

    pub fn content_height(&self) -> f32 {
        self.height - self.margin_top - self.margin_bottom
    }

    pub fn top(&self) -> f32 {
        self.margin_top
    }

    pub fn bottom(&self) -> f32 {
        self.height - self.margin_bottom
    }

    pub fn left(&self) -> f32 {
        self.margin_left
    }

    pub fn right(&self) -> f32 {
        self.width - self.margin_right
    }
}
