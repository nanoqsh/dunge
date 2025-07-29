use std::borrow::Cow;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Describes a layout for user types.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Define<T>(Cow<'static, [T]>)
where
    T: Clone + 'static;

impl<T> Define<T>
where
    T: Clone,
{
    /// Creates a new definition of a type.
    pub const fn new(s: &'static [T]) -> Self {
        Self(Cow::Borrowed(s))
    }

    /// Returns the definition length.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks is definition empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns an item by given `index`.
    pub fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }

    /// Iterate over all items.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = T>
    where
        T: Copy,
    {
        self.0.iter().copied()
    }
}
