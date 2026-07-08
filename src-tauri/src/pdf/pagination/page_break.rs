use super::{
    constants::{FLOAT_EPSILON, MIN_REMAIN_HEIGHT},
    PageContext,
};

pub struct PageBreak;

impl PageBreak {
    /// Chiều cao còn lại của page.
    #[inline]
    pub fn remaining_height(current_y: f32, page: &PageContext) -> f32 {
        (page.content_height() - current_y).max(0.0)
    }

    /// Kiểm tra còn đủ để render element.
    #[inline]
    pub fn can_fit(current_y: f32, element_height: f32, page: &PageContext) -> bool {
        Self::remaining_height(current_y, page) + FLOAT_EPSILON >= element_height
    }

    /// Có cần chuyển trang hay không.
    #[inline]
    pub fn should_break(current_y: f32, element_height: f32, page: &PageContext) -> bool {
        !Self::can_fit(current_y, element_height, page)
    }

    /// Kiểm tra còn đủ khoảng trống tối thiểu.
    #[inline]
    pub fn has_remaining_space(current_y: f32, page: &PageContext) -> bool {
        Self::remaining_height(current_y, page) >= MIN_REMAIN_HEIGHT
    }

    /// Y bắt đầu của content ở trang mới.
    #[inline]
    pub fn first_y(page: &PageContext) -> f32 {
        page.margin_top
    }
}
