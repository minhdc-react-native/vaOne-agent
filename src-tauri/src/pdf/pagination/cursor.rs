#[derive(Debug, Clone, Default)]
pub struct Cursor {
    /// Trang hiện tại (bắt đầu từ 0)
    pub page_index: usize,

    /// Vị trí Y hiện tại trong page (pixel)
    pub current_y: f32,
}

impl Cursor {
    pub fn new(start_y: f32) -> Self {
        Self {
            page_index: 0,
            current_y: start_y,
        }
    }

    /// Sang trang mới
    pub fn next_page(&mut self, start_y: f32) {
        self.page_index += 1;
        self.current_y = start_y;
    }

    /// Di chuyển xuống dưới
    pub fn move_down(&mut self, height: f32) {
        self.current_y += height;
    }

    /// Đặt lại vị trí Y
    pub fn set_y(&mut self, y: f32) {
        self.current_y = y;
    }

    /// Reset paginator
    pub fn reset(&mut self, start_y: f32) {
        self.page_index = 0;
        self.current_y = start_y;
    }
}
