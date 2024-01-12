use std::{iter, slice};

pub struct Define<T>(&'static [T])
where
    T: 'static;

impl<T> Define<T> {
    pub const fn new(s: &'static [T]) -> Self {
        Self(s)
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
