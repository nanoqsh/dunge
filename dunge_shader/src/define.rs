use std::{iter, slice};

/// Describes a layout for user types.
pub struct Define<T>(&'static [T])
where
    T: 'static;

impl<T> Define<T> {
    /// Creates a new definition of a type.
    pub const fn new(s: &'static [T]) -> Self {
        Self(s)
    }

    /// Returns the definition length.
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks is definition empty.
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an item by given `index`.
    pub fn get(&self, index: usize) -> Option<&'static T> {
        self.0.get(index)
    }
}

impl<T> Clone for Define<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Define<T> {}

impl<T> IntoIterator for Define<T>
where
    T: Copy,
{
    type Item = T;
    type IntoIter = iter::Copied<slice::Iter<'static, Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}
