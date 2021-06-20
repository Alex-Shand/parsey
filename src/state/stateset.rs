use std::fmt;

use syntax_abuse as syntax;

use super::item::Item;

/// The set of Earley items produced from one step of the algorithm
#[derive(PartialEq, Clone, Debug)]
pub(crate) struct StateSet<'a> {
    items: Vec<Item<'a>>,
    next: usize,
}

impl<'a> StateSet<'a> {
    /// Constructs a new state set from a vector of items. Note: This function
    /// assumes that each item in the vector is unique, though it's probably
    /// harmless if that isn't true the parser will do redundant work if there
    /// are duplicates.
    pub(crate) fn new(items: Vec<Item<'a>>) -> Self {
        StateSet { items, next: 0 }
    }

    #[cfg(test)]
    pub(crate) fn exhausted(items: Vec<Item<'a>>) -> Self {
        let next = items.len() + 1;
        StateSet { items, next }
    }

    syntax::get! { pub items : [Item<'a>] }

    /// Effectively `Iterator::next`. No point using an actual iterator because
    /// a for loop won't work while building the state set (as we have to mutate
    /// it while iterating) and the rest of the time we operate on the whole set
    /// at once so can just get a reference to the underlying vector of items.
    pub(crate) fn next(&mut self) -> Option<Item<'a>> {
        let current = self.next;
        self.next += 1;
        self.items.get(current).copied()
    }

    /// Add a bunch of new items to the state set, checking for each whether it
    /// is already there.
    pub(crate) fn add(&mut self, new_items: Vec<Item<'a>>) {
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
            self.items
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

syntax::tests! {
    use crate::rule;
    use crate::grammar::Rule;

    testdata! {
        RULE : Rule = rule! {
            Rule -> "Rule"
        };
    }

    testcase! {
        new_doesnt_check_for_duplicates,
        StateSet::new(Item::from_rules(vec![&RULE, &RULE], 0)),
        StateSet { items: Item::from_rules(vec![&RULE, &RULE], 0), next: 0 }
    }

    #[test]
    fn add_does_check_for_duplicates() {
        let mut state = StateSet::new(Item::from_rules(vec![&RULE], 0));
        let orig_state = state.clone();
        state.add(Item::from_rules(vec![&RULE], 0));
        assert_eq!(state, orig_state)
    }

    #[test]
    fn next() {
        let rules = vec![
            rule! { Rule1 -> "Rule1" },
            rule! { Rule2 -> "Rule2" },
            rule! { Rule3 -> "Rule3" }
        ];
        let items = Item::from_rules(rules.iter().collect::<Vec<_>>(), 0);
        let items2 = items.clone();
        let mut state = StateSet::new(items);
        assert_eq!(state.next(), Some(items2[0]));
        assert_eq!(state.next(), Some(items2[1]));
        assert_eq!(state.next(), Some(items2[2]));
    }
}
