#[derive(Debug, Clone, Copy)]
pub struct MetaContainer<T> {
    item: T,
    line: usize
}

impl<T> MetaContainer<T> {
    pub fn new(item: T, line: usize) -> Self {
        MetaContainer { item, line }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn item<'a>(&'a self)-> &'a T {
        &self.item
    }
}

impl<T> MetaContainer<T> where T: Copy {
    pub fn item_copy(&self)-> T {
        self.item
    }
}

impl<T> MetaContainer<T> where T: Clone {
    pub fn item_clone(&self)-> T {
        self.item.clone()
    }
}
