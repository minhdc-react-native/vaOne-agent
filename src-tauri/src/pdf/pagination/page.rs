use super::paginator::PageItem;
/// Kết quả của một trang.
#[derive(Debug, Clone)]
pub struct PageLayout {
    pub index: usize,
    pub items: Vec<PageItem>,
}

impl PageLayout {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            items: Vec::new(),
        }
    }

    pub fn push(&mut self, item: PageItem) {
        self.items.push(item);
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}
