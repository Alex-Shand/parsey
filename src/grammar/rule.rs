use super::symbol::Symbol;

use syntax_abuse as syntax;

/// [Grammar](super::Grammar) rule
#[derive(Debug, PartialEq)]
pub struct Rule {
    name: String,
    body: Vec<Symbol>
}

impl Rule {
    /// Construct a new rule with a specific name and body. Rule names cannot
    /// begin with the `@` character.
    ///
    /// # Panics
    /// If the rule name begins with `@`
    pub fn new(name: String, body: Vec<Symbol>) -> Self {
        assert!(
            !name.starts_with("@"),
            "Rule names beginning with @ are reserved"
        );
        Rule { name, body }
    }

    syntax::getter! { pub name : &str }

    pub fn get(&self, index: usize) -> Option<&Symbol> {
        self.body.get(index)
    }
}

syntax::tests! {

    #[test]
    #[should_panic]
    fn reserved_name() {
        Rule::new(String::from("@reserved"), vec![]);
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
                Symbol::OneOf(vec!['o', 'n', 'e', 'o', 'f']),
                Symbol::Rule(String::from("Rule"))
            ]
        }
    }
}
