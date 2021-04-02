use std::fmt;
    
use syntax_abuse as syntax;

use super::item::Item;
    
/// The set of Earley items produced from one step of the algorithm
#[derive(Debug)]
pub struct StateSet<'a> {
    items: Vec<Item<'a>>,
    next: usize
}

impl<'a> StateSet<'a> {

    /// Constructs a new state set from a vector of items. Note: This function
    /// assumes that each item in the vector is unique, though it's probably
    /// harmless if that isn't true the parser will do redundant work if there
    /// are duplicates.
    pub fn new(items: Vec<Item<'a>>) -> Self {
        StateSet { items, next: 0 }
    }

    syntax::get! { pub items : [Item<'a>] }
    
    /// Effectively Iterator::next. No point using an actual iterator because a
    /// for loop won't work while building the state set (as we have to mutate
    /// it while iterating) and the rest of the time we operate on the whole set
    /// at once so can just get a reference to the underlying vector of items.
    pub fn next(&mut self) -> Option<&Item<'a>> {
        let current = self.next;
        self.next += 1;
        self.items.get(current)
    }

    /// Add a bunch of new items to the state set, checking for each whether it
    /// is already there.
    pub fn add(&mut self, new_items: Vec<Item<'a>>) {
        for item in new_items {
            if !self.items.contains(&item) {
                self.items.push(item)
            }
        }
    }
}

impl fmt::Display for StateSet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.items.iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
