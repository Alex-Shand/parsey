pub use rule::Rule;
pub use symbol::Symbol;

#[macro_use]
mod macros;
mod rule;
mod symbol;

/// Grammar suitable for Earley parsing
#[derive(Debug, PartialEq)]
pub struct Grammar(Vec<Rule>);

impl Grammar {
    /// Construct a new grammar from a list of rules. The first rule in the list
    /// is the root rule. Alternations (`A -> B | C`) are not supported, instead
    /// create several rules with the same name (`A -> B` and `A -> C`)
    ///
    /// # Panics
    /// If the rule list is empty
    pub fn new(rules: Vec<Rule>) -> Self {
        assert!(!rules.is_empty(), "A grammar must have at least one rule");
        Grammar(rules)
    }

    pub fn recognise<S>(&self, input: S) -> bool where S: AsRef<str> {
        super::recognise(&self, input)
    }
    
    pub fn start_symbol(&self) -> &str {
        &self.0[0].name()
    }

    pub fn get_rules_by_name(&self, name: &str) -> Vec<&Rule> {
        self.0.iter().filter(|rule| rule.name() == name).collect::<Vec<_>>()
    }
}

syntax_abuse::tests! {
    #[test]
    #[should_panic]
    fn empty_rules() {
        Grammar::new(vec![]);
    }

    testcase! {
        non_empty_rules,
        Grammar::new(vec![Rule::new(String::from("Test"), vec![])]),
        Grammar(vec![Rule::new(String::from("Test"), vec![])])
    }

    testcase! {
        grammar_macro,
        grammar! {
            Rule -> Rule2;
            Rule2 -> "literal";
        },
        Grammar(vec![
            Rule::new(
                String::from("Rule"),
                vec![Symbol::Rule(String::from("Rule2"))]
            ),
            Rule::new(
                String::from("Rule2"),
                vec![
                    Symbol::Literal('l'),
                    Symbol::Literal('i'),
                    Symbol::Literal('t'),
                    Symbol::Literal('e'),
                    Symbol::Literal('r'),
                    Symbol::Literal('a'),
                    Symbol::Literal('l')
                ]
            )
        ])
    }

    testcase! {
        trailing_semi_is_optional,
        grammar! {
            Rule -> Rule2;
            Rule2 -> "literal"
        },
        Grammar(vec![
            Rule::new(
                String::from("Rule"),
                vec![Symbol::Rule(String::from("Rule2"))]
            ),
            Rule::new(
                String::from("Rule2"),
                vec![
                    Symbol::Literal('l'),
                    Symbol::Literal('i'),
                    Symbol::Literal('t'),
                    Symbol::Literal('e'),
                    Symbol::Literal('r'),
                    Symbol::Literal('a'),
                    Symbol::Literal('l')
                ]
            )
        ])
    }

    testcase! {
        only_one_rule,
        grammar! {
            Rule -> "literal"
        },
        Grammar(vec![
            Rule::new(
                String::from("Rule"),
                vec![
                    Symbol::Literal('l'),
                    Symbol::Literal('i'),
                    Symbol::Literal('t'),
                    Symbol::Literal('e'),
                    Symbol::Literal('r'),
                    Symbol::Literal('a'),
                    Symbol::Literal('l')
                ]
            )
        ])
    }

    testcase! {
        realish_grammar,
        grammar! {
            Sum -> Sum ["+-"] Product;
            Sum -> Product;
            Product -> Product ["*/"] Factor;
            Product -> Factor;
            Factor -> "(" Sum ")";
            Factor -> Number;
            Number -> ["0123456789"] Number;
            Number -> ["0123456789"];
        },
        Grammar(vec![
            Rule::new(
                String::from("Sum"),
                vec![
                    Symbol::Rule(String::from("Sum")),
                    Symbol::OneOf(hashset!['+', '-']),
                    Symbol::Rule(String::from("Product"))
                ]
            ),
            Rule::new(
                String::from("Sum"),
                vec![
                    Symbol::Rule(String::from("Product"))
                ]
            ),
            Rule::new(
                String::from("Product"),
                vec![
                    Symbol::Rule(String::from("Product")),
                    Symbol::OneOf(hashset!['*', '/']),
                    Symbol::Rule(String::from("Factor"))
                ]
            ),
            Rule::new(
                String::from("Product"),
                vec![
                    Symbol::Rule(String::from("Factor"))
                ]
            ),
            Rule::new(
                String::from("Factor"),
                vec![
                    Symbol::Literal('('),
                    Symbol::Rule(String::from("Sum")),
                    Symbol::Literal(')')
                ]
            ),
            Rule::new(
                String::from("Factor"),
                vec![
                    Symbol::Rule(String::from("Number"))
                ]
            ),
            Rule::new(
                String::from("Number"),
                vec![
                    Symbol::OneOf(hashset![
                        '0',
                        '1',
                        '2',
                        '3',
                        '4',
                        '5',
                        '6',
                        '7',
                        '8',
                        '9'
                    ]),
                    Symbol::Rule(String::from("Number"))
                ]
            ),
            Rule::new(
                String::from("Number"),
                vec![
                    Symbol::OneOf(hashset![
                        '0',
                        '1',
                        '2',
                        '3',
                        '4',
                        '5',
                        '6',
                        '7',
                        '8',
                        '9'
                    ])
                ]
            )
        ])
    }
}
