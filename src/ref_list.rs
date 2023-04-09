#![allow(dead_code)]

/// A linked list of references to `RefList`s.
/// It is useful for keeping track of a function call stack, e.g.
/// when recursively searching for cyclical dependencies.
#[derive(Debug, Clone, Copy)]
pub struct RefList<'a, T> {
    item: &'a T,
    base: &'a T,
    prev: Option<&'a RefList<'a, T>>,
}

impl<'a, T> RefList<'a, T> {
    pub fn new(item: &'a T) -> Self {
        Self {
            item,
            prev: None,
            base: item,
        }
    }

    pub fn push(&'a self, item: &'a T) -> Self {
        Self {
            item,
            prev: Some(self),
            base: self.base,
        }
    }

    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        if self.item == item {
            true
        } else if let Some(prev) = self.prev {
            prev.contains(item)
        } else {
            false
        }
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = &'a T> {
        std::iter::successors(Some(self), |prev| prev.prev).map(|prev| prev.item)
    }

    pub fn top(&self) -> &'a T {
        self.item
    }

    pub fn base(&self) -> &'a T {
        self.base
    }
}

/// A linked list of references to `RefList`s, which can contain zero values.
/// It is useful for keeping track of a function call stack, e.g.
/// when recursively searching for cyclical dependencies.
#[derive(Debug, Clone, Copy)]
pub struct ERefList<'a, T> {
    list: Option<RefList<'a, T>>,
}

impl<'a, T> ERefList<'a, T> {
    pub fn new() -> Self {
        Self { list: None }
    }

    pub fn push(&'a self, item: &'a T) -> Self {
        Self {
            list: Some(
                self.list
                    .as_ref()
                    .map(|l| l.push(item))
                    .unwrap_or_else(|| RefList::new(item)),
            ),
        }
    }

    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        if let Some(list) = &self.list {
            list.contains(item)
        } else {
            false
        }
    }

    pub fn iter(&self) -> impl '_ + Iterator<Item = &'a T> {
        self.list.iter().flat_map(|list| list.iter())
    }

    pub fn top(&self) -> Option<&'a T> {
        self.list.as_ref().map(|list| list.top())
    }

    pub fn base(&self) -> Option<&'a T> {
        self.list.as_ref().map(|list| list.base())
    }
}
