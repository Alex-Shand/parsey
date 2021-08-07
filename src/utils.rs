use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Sub;

use derive_deref::Deref;

/// `HashSet` which always has at least one item
#[derive(Debug, PartialEq, Clone, Deref)]
pub struct NonEmptyHashSet<T>
where
    HashSet<T>: PartialEq,
{
    contents: HashSet<T>,
}

impl<T> NonEmptyHashSet<T>
where
    T: Hash + Eq,
{
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

#[derive(Debug, Copy, Clone)]
pub(crate) enum Uncertain {
    Known(usize),
    Unknown(usize),
}

impl Sub for Uncertain {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Uncertain::Known(a) => match rhs {
                Uncertain::Known(b) => Uncertain::Known(a - b),
                Uncertain::Unknown(b) => Uncertain::Unknown(a - b),
            },
            Uncertain::Unknown(a) => {
                let b = match rhs {
                    Uncertain::Known(b) | Uncertain::Unknown(b) => b
                };
                Uncertain::Unknown(a - b)
            },
        }
    }
}
