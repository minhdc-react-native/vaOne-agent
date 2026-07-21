/// ===============================
/// Default Page Margin
/// Đơn vị: pixel
/// ===============================

pub const DEFAULT_MARGIN_TOP: f32 = 24.0 * 1.5;
pub const DEFAULT_MARGIN_RIGHT: f32 = 24.0;
pub const DEFAULT_MARGIN_BOTTOM: f32 = 24.0 * 1.5;
pub const DEFAULT_MARGIN_LEFT: f32 = 24.0;

/// Chiều cao tối thiểu còn lại để tiếp tục render.
///
/// Nếu nhỏ hơn giá trị này thì xem như trang đã đầy.
pub const MIN_REMAIN_HEIGHT: f32 = 2.0;

/// Sai số dùng khi so sánh số thực.
pub const FLOAT_EPSILON: f32 = 0.01;
