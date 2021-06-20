use std::collections::HashSet;
use std::fmt;

use super::symbol::Symbol;

use syntax_abuse as syntax;

/// [Grammar](super::Grammar) rule
#[derive(Clone, PartialEq)]
pub struct Rule {
    name: String,
    body: Vec<Symbol>,
}

impl Rule {
    /// Construct a new rule with a specific name and body. Rule names cannot
    /// begin with the `@` character.
    ///
    /// # Panics
    /// If the rule name begins with `@`
    #[must_use]
    pub fn new(name: String, body: Vec<Symbol>) -> Self {
        assert!(
            !name.starts_with('@'),
            "Rule names beginning with @ are reserved"
        );
        Rule { name, body }
    }

    syntax::get! { pub(crate) name : str }
    syntax::get! { pub(crate) body : [Symbol] }

    pub(crate) fn get(&self, index: usize) -> Option<&Symbol> {
        self.body.get(index)
    }

    pub(crate) fn is_nullable(&self, nullable_symbols: &HashSet<String>) -> bool {
        if self.body.is_empty() {
            return true;
        }

        if self.body.iter().any(Symbol::is_terminal) {
            return false;
        }

        return self
            .body
            .iter()
            .all(|s| nullable_symbols.contains(s.rule_name().unwrap()));
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {}",
            self.name,
            self.body
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

syntax::tests! {

    #[test]
    #[should_panic]
    fn reserved_name() {
        drop(Rule::new(String::from("@reserved"), vec![]));
    }

    testcase! {
        valid_rule,
        Rule::new(String::from("Rule"), vec![]),
        Rule { name: String::from("Rule"), body: vec![] }
    }

    testcase! {
        rule_macro,
        rule!(Rule -> "literal" ["oneof"] Rule),
        Rule {
            name: String::from("Rule"),
            body: vec![
                Symbol::Literal('l'),
                Symbol::Literal('i'),
                Symbol::Literal('t'),
                Symbol::Literal('e'),
                Symbol::Literal('r'),
                Symbol::Literal('a'),
                Symbol::Literal('l'),
                Symbol::OneOf(hashset!['o', 'n', 'e', 'o', 'f']),
                Symbol::Rule(String::from("Rule"))
            ]
        }
    }
}
