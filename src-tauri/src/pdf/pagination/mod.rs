/*
pagination/
│
├── mod.rs
├── constants.rs
├── cursor.rs
├── page.rs
├── page_context.rs
├── page_break.rs
├── paginator.rs
└── page_renderer.rs
*/

//! Pagination engine.
//!
//! Chịu trách nhiệm:
//! - Chia report thành nhiều trang
//! - Tính toán vị trí các element trong từng trang
//! - Chia TableLayoutResult thành nhiều phần theo page
//! - Không phụ thuộc printpdf
//!
//! Luồng xử lý:
//!
//! Layout
//!     ↓
//! Paginator
//!     ↓
//! Vec<PageLayout>
//!     ↓
//! PageRenderer
//!     ↓
//! PdfPage

pub mod constants;
pub mod cursor;
pub mod layout_builder;
pub mod page;
pub mod page_break;
pub mod page_context;
pub mod page_renderer;
pub mod pagination_context;
pub mod paginator;
// Re-export các kiểu thường dùng
pub use cursor::Cursor;
pub use page::PageLayout;
pub use page_context::PageContext;
pub use page_renderer::PageRenderer;
pub use paginator::{PageItem, Paginator};
