use crate::pdf::models::Element;

/// Một trang sau khi phân trang
#[derive(Debug, Default)]
pub struct PageLayout {
    pub number: usize,
    pub elements: Vec<Element>,
}

/// Một phần của Table sau khi được chia trang
///
/// Một TableLayoutResult có thể tạo thành nhiều TablePage.
#[derive(Debug, Clone)]
pub struct TablePage {
    /// Vị trí bắt đầu của table trên trang
    pub x: f32,

    pub y: f32,

    pub width: f32,

    /// Chiều cao thực tế của fragment
    pub height: f32,

    /// Header được lặp lại ở mỗi trang
    pub headers: Vec<TableRowLayout>,

    /// Các dòng dữ liệu thuộc fragment này
    pub rows: Vec<TableRowLayout>,
}

impl TablePage {
    pub fn new(x: f32, y: f32, width: f32) -> Self {
        Self {
            x,
            y,
            width,
            height: 0.0,
            headers: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Tổng chiều cao header
    pub fn header_height(&self) -> f32 {
        self.headers.iter().map(|r| r.height).sum()
    }

    /// Tổng chiều cao body
    pub fn body_height(&self) -> f32 {
        self.rows.iter().map(|r| r.height).sum()
    }

    /// Chiều cao toàn bộ fragment
    pub fn total_height(&self) -> f32 {
        self.header_height() + self.body_height()
    }
}

/// Tiện ích dùng khi tính lại vị trí các Row
pub struct RowPositioner;

impl RowPositioner {
    pub fn relocate(page: &mut TablePage) {
        let mut current_y = page.y;

        // Header
        for row in &mut page.headers {
            row.y = current_y;

            for cell in &mut row.cells {
                cell.y = current_y;
            }

            current_y += row.height;
        }

        // Body
        for row in &mut page.rows {
            row.y = current_y;

            for cell in &mut row.cells {
                cell.y = current_y;
            }

            current_y += row.height;
        }

        page.height = current_y - page.y;
    }
}
