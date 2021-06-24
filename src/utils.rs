use std::collections::HashSet;
use std::hash::Hash;

use derive_deref::Deref;

/// `HashSet` which always has at least one item
#[derive(Debug, PartialEq, Clone, Deref)]
pub struct NonEmptyHashSet<T> where HashSet<T>: PartialEq {
    contents: HashSet<T>
}

impl<T> NonEmptyHashSet<T> where T: Hash + Eq {
    /// Wrap a `HashSet` in a `NonEmptyHashSet`
    ///
    /// # Panics
    /// If the input `HashSet` is empty
    #[must_use]
    pub fn new(contents: HashSet<T>) -> Self {
        assert!(!contents.is_empty(), "NonEmptyHashSet must not be empty");
        Self { contents }
    }
}
