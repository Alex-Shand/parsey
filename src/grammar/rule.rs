use super::{ Matcher, State, Item };

use syntax_abuse as syntax;

/// Parses a single rule (without the trailing ;) on behalf of grammar! { }
#[macro_export]
#[doc(hidden)]
macro_rules! rule {
    // Rule syntax is RuleName -> body
    // The rule name is a bareword as in matcher!() above.
    // Conveniently (from matcher!()) all of the possible matcher syntaxes are
    // parsed as a single token tree so the rule body can be a (possibly empty)
    // list of token trees.
    ($name:ident -> $($matchers:tt)*) => {
        $crate::grammar::Rule::new(
            String::from(::std::stringify!($name)),
            vec![
                // matcher! is used to parse each token tree in the body, the
                // result is a Vec<Vec<Matcher>> which has to be flattened for
                // Rule::new
                $($crate::matcher!($matchers)),*
            ].into_iter().flatten().collect::<Vec<_>>()
        )
    }
}

/// [Grammar] rule
#[derive(Debug, PartialEq)]
pub struct Rule {
    name: String,
    body: Vec<Matcher>
}

impl Rule {
    /// Construct a new rule with a specific name and body. Rule names cannot
    /// begin with the `@` character.
    ///
    /// # Panics
    /// If the rule name begins with `@`
    pub fn new(name: String, body: Vec<Matcher>) -> Self {
        assert!(
            !name.starts_with("@"),
            "Rule names beginning with @ are reserved"
        );
        Rule { name, body }
    }

    syntax::getter! { pub name : &str }

    pub fn get(&self, index: usize) -> Option<&Matcher> {
        self.body.get(index)
    }
    
    pub fn to_earley_item(&self, state: State) -> Item<'_> {
        assert!(
            state.progress <= self.body.len(),
            "Progress is {} but the rule only has {} items",
            state.progress,
            self.body.len()
        );
        Item { rule: &self, state: state }
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
                Matcher::Literal('l'),
                Matcher::Literal('i'),
                Matcher::Literal('t'),
                Matcher::Literal('e'),
                Matcher::Literal('r'),
                Matcher::Literal('a'),
                Matcher::Literal('l'),
                Matcher::OneOf(vec!['o', 'n', 'e', 'o', 'f']),
                Matcher::Rule(String::from("Rule"))
            ]
        }
    }
}
