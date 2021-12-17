use std::ops::Deref;

#[derive(Debug)]
pub struct NTree<U> {
    data: U,
    children: Vec<NTree<U>>
}

impl<U> Deref for NTree<U> {
    type Target = U;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<U> NTree<U> {
    pub fn new(data: U) -> Self {
        NTree { data, children: vec!() }
    }

    pub fn insert(&mut self, child: Self) {
        self.children.push(child);
    }

    pub fn children(&self) -> &Vec<NTree<U>> {
        &self.children
    }
}
