use std::fmt;

use syntax_abuse as syntax;

use super::{
    set::StateSet,
    super::grammar::{ Grammar, Rule, Symbol }
};
    
#[derive(PartialEq, Copy, Clone)]
pub struct Item<'a> {
    rule: &'a Rule,
    start: usize,
    progress: usize
}

impl<'a> Item<'a> {
    /// Construct a vector of items from a vector of rules, each item starts
    /// at the given start position and has its progress marker set to 0.
    pub fn from_rules(rules: Vec<&'a Rule>, start: usize) -> Vec<Self> {
        rules.into_iter()
            .map(|rule| Item { rule, start, progress: 0 })
            .collect::<Vec<_>>()
    }

    syntax::get! { pub start : usize }

    /// The name of the rule this item wraps.
    pub fn rule_name(&self) -> &str {
        &self.rule.name()
    }

    /// True if the item is complete
    pub fn is_complete(&self) -> bool {
        self.progress >= self.rule.body().len()
    }

    /// Perform the relevant step from the earley algorithm for the current
    /// item. Predictions and Completions mutate current_state directly,
    /// Scans return the rule to be added to the next set (if applicable)
    /// for the caller to make use of.
    pub fn parse(
        &self,
        grammar: &'a Grammar,
        current_state: &mut StateSet<'a>,
        prev_state: &[StateSet<'a>],
        input: &[char],
        current_position: usize
    ) -> Option<Item<'a>> {
        if let Some(matcher) = self.rule.get(self.progress) {
            match matcher {
                Symbol::Rule(name) => {
                    // Prediction: Add all rules that can produce the
                    // required non-terminal to the current state set,
                    // starting from the current position
                    current_state.add(
                        Item::from_rules(
                            grammar.get_rules_by_name(name),
                            current_position
                        )
                    );
                    None
                }
                // Scan: If the current character matches the current
                // non-terminal (criteria differs per terminal) then return
                // the current item advanced by one place (over the
                // terminal), this will be added to the next state set by
                // the caller when it is created.
                Symbol::Literal(c) => self.scan(
                    input,
                    current_position,
                    |next| next == c
                ),
                Symbol::OneOf(cs) => self.scan(
                    input,
                    current_position,
                    |next| cs.contains(next)
                )
            }
        } else {
            // Completion: Find all rules in the state set this item started
            // in that need the non-terminal produced by this rule to
            // complete and add them to this state set advanced by one place
            // (over the non-terminal)
            let completed = self.rule.name();
            current_state.add(
                prev_state[self.start].items().iter()
                    .filter_map(|item| {
                        item.next_name()
                            .filter(|name| *name == completed)
                            .map(|_| item.advanced())
                    }).collect::<Vec<Item>>()
            );
            None
        }
    }

    /// Common Scan implementation. Unconditionally returns None if `pos` is
    /// past the end of `input`, otherwise the character is passed to pred. If
    /// pred succeeds the item is returned advanced by one place (see the Scan
    /// branch of Item::parse), if it fails None is returned.
    fn scan(
        &self,
        input: &[char],
        pos: usize,
        pred: impl FnOnce(&char) -> bool
    ) -> Option<Self> {
        input.get(pos).copied().filter(pred).map(|_| self.advanced())
    }

    /// If the next symbol to be processed is a rule this returns the name of
    /// that rule, otherwise it returns None.
    fn next_name(&self) -> Option<&str> {
        if let Some(symbol) = self.rule.get(self.progress) {
            match symbol {
                Symbol::Rule(name) => Some(name),
                _ => None
            }
        } else {
            None
        }
    }

    /// Returns a copy of the current item with its progress marker advanced
    /// one step
    fn advanced(&self) -> Self {
        let mut new = *self;
        new.progress += 1;
        new
    }
}

impl fmt::Display for Item<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut body = self.rule.body().to_owned();

        body.insert(self.progress, Symbol::Rule(String::from("\u{25CF}")));

        let body = body.into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>().join(" ");

        write!(f, "{} -> {} ({})", self.rule.name(), body, self.start)
    }
}

impl fmt::Debug for Item<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

syntax::tests! {
    use crate::rule;
    use crate::grammar::Rule;

    testdata! {
        RULE  : Rule = rule! { Rule  -> "Rule"    };
        RULE2 : Rule = rule! { Rule2 -> "Rule2"   };
        RULE3 : Rule = rule! { Rule3 -> "Rule3"   };
        RULE4 : Rule = rule! { Rule4 -> ["Rule4"] };
        RULE5 : Rule = rule! { Rule5 -> Rule      };
    }

    testcase! {
        from_rules,
        Item::from_rules(vec![&RULE, &RULE2, &RULE3], 1),
        vec![
            Item { rule: &RULE,  start: 1, progress: 0 },
            Item { rule: &RULE2, start: 1, progress: 0 },
            Item { rule: &RULE3, start: 1, progress: 0 }
        ]
    }

    testcase! {
        rule_name,
        Item { rule: &RULE, start: 0, progress: 0 }.rule_name(),
        "Rule"
    }

    testcase! {
        is_complete_completed,
        Item { rule: &RULE, start: 0, progress: 4 }.is_complete(),
        true
    }

    testcase! {
        is_complete_overshoot,
        Item { rule: &RULE, start: 0, progress: 300 }.is_complete(),
        true
    }

    testcase! {
        is_complete_incomplete,
        Item { rule: &RULE, start: 0, progress: 0 }.is_complete(),
        false
    }

    testcase! {
        next_name_literal,
        Item { rule: &RULE, start: 0, progress: 0 }.next_name(),
        None
    }

    testcase! {
        next_name_oneof,
        Item { rule: &RULE4, start: 0, progress: 0 }.next_name(),
        None
    }

    testcase! {
        next_name_rule,
        Item { rule: &RULE5, start: 0, progress: 0 }.next_name(),
        Some("Rule")
    }

    testcase! {
        advanced,
        Item { rule: &RULE, start: 0, progress: 0 }.advanced(),
        Item { rule: &RULE, start: 0, progress: 1 }
    }

    #[test]
    fn parse_rule() {
        let rule = rule! { Rule -> Rule2 };
        let rule2 = rule! { Rule2 -> "Rule2" };
        let grammar = Grammar::new(vec![rule.clone(), rule2.clone()]);
        let mut state = StateSet::new(vec![]);
        let prev = Vec::new();
        let input = Vec::new();
        assert_eq!(
            Item { rule: &rule, start: 0, progress: 0 }.parse(
                &grammar,
                &mut state,
                &prev,
                &input,
                0
            ),
            None
        );
        assert_eq!(state.items(), Item::from_rules(vec![&rule2], 0))
    }

    #[test]
    fn parse_literal_success() {
        let rule = rule! { Rule -> "x" };
        let grammar = Grammar::new(vec![rule.clone()]);
        let mut state = StateSet::new(vec![]);
        let prev = Vec::new();
        let input = vec!['x'];
        assert_eq!(
            Item { rule: &rule, start: 0, progress: 0 }.parse(
                &grammar,
                &mut state,
                &prev,
                &input,
                0
            ),
            Some(Item { rule: &rule, start: 0, progress: 1 })
        );
        assert_eq!(state.items(), vec![])
    }

    #[test]
    fn parse_literal_failure() {
        let rule = rule! { Rule -> "y" };
        let grammar = Grammar::new(vec![rule.clone()]);
        let mut state = StateSet::new(vec![]);
        let prev = Vec::new();
        let input = vec!['x'];
        assert_eq!(
            Item { rule: &rule, start: 0, progress: 0 }.parse(
                &grammar,
                &mut state,
                &prev,
                &input,
                0
            ),
            None
        );
        assert_eq!(state.items(), vec![])
    }

    #[test]
    fn parse_completion() {
        let rule = rule! { Rule -> Rule2 };
        let rule2 = rule! { Rule2 -> "Rule2" };
        let grammar = Grammar::new(vec![rule.clone(), rule2.clone()]);
        let mut state = StateSet::new(vec![]);
        let prev = vec![StateSet::new(Item::from_rules(vec![&rule], 0))];
        let input = Vec::new();
        assert_eq!(
            Item { rule: &rule2, start: 0, progress: 5 }.parse(
                &grammar,
                &mut state,
                &prev,
                &input,
                0
            ),
            None
        );
        assert_eq!(
            state.items(),
            vec![Item { rule: &rule, start: 0, progress: 1 }]
        )
    }
}
