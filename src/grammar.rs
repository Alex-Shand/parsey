//!
use std::collections::HashSet;
use std::fmt;

pub use rule::Rule;
pub use symbol::Symbol;

#[macro_use]
mod macros;
mod rule;
mod symbol;

/// Grammar suitable for Earley parsing
#[derive(Debug, PartialEq)]
pub struct Grammar {
    rules: Vec<Rule>,
    nullables: HashSet<String>,
}

impl Grammar {
    /// Construct a new grammar from a list of rules. The first rule in the list
    /// is the root rule. Alternations (`A -> B | C`) are not supported, instead
    /// create several rules with the same name (`A -> B` and `A -> C`)
    ///
    /// # Panics
    /// If the rule list is empty
    #[must_use]
    pub fn new(rules: Vec<Rule>) -> Self {
        assert!(!rules.is_empty(), "A grammar must have at least one rule");
        let nullables = find_nullable_rules(&rules);
        Grammar { rules, nullables }
    }

    pub(crate) fn start_symbol(&self) -> &str {
        &self.rules[0].name()
    }

    pub(crate) fn get_rules_by_name(&self, name: &str) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|rule| rule.name() == name)
            .collect::<Vec<_>>()
    }

    pub(crate) fn rule_is_nullable(&self, rule: &str) -> bool {
        self.nullables.contains(rule)
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn index(&self, idx: usize) -> &Rule {
        &self.rules[idx]
    }
}

impl fmt::Display for Grammar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.rules
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

fn find_nullable_rules(rules: &[Rule]) -> HashSet<String> {
    let mut nullables = HashSet::new();
    let mut count = 0;
    for rule in rules {
        if rule.is_nullable(&nullables) {
            let _ = nullables.insert(rule.name().to_owned());
        }
    }

    while count < nullables.len() {
        count = nullables.len();
        for rule in rules {
            if rule.is_nullable(&nullables) {
                let _ = nullables.insert(rule.name().to_owned());
            }
        }
    }

    nullables
}

syntax_abuse::tests! {
    #[test]
    #[should_panic]
    fn empty_rules() {
        drop(Grammar::new(vec![]));
    }

    testcase! {
        non_empty_rules,
        Grammar::new(vec![Rule::new(String::from("Test"), vec![])]),
        Grammar {
            rules: vec![Rule::new(String::from("Test"), vec![])],
            nullables: hashset![String::from("Test")]
        }
    }

    testcase! {
        grammar_macro,
        grammar! {
            Rule -> Rule2;
            Rule2 -> "literal";
        },
        Grammar {
            rules: vec![
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
            ],
            nullables: hashset![]
        }
    }

    testcase! {
        trailing_semi_is_optional,
        grammar! {
            Rule -> Rule2;
            Rule2 -> "literal"
        },
        Grammar {
            rules: vec![
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
            ],
            nullables: hashset![]
        }
    }

    testcase! {
        only_one_rule,
        grammar! {
            Rule -> "literal"
        },
        Grammar {
            rules: vec![
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
            ],
            nullables: hashset![]
        }
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
        Grammar {
            rules: vec![
                Rule::new(
                    String::from("Sum"),
                    vec![
                        Symbol::Rule(String::from("Sum")),
                        Symbol::OneOf(nonempty_hashset!['+', '-']),
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
                        Symbol::OneOf(nonempty_hashset!['*', '/']),
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
                        Symbol::OneOf(nonempty_hashset![
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
                        Symbol::OneOf(nonempty_hashset![
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
            ],
            nullables: hashset![]
        }
    }


    testdata! {
        NULLABILITY: Grammar = grammar! {
            TriviallyNullable -> ;
            OnlyUsesNullableRules -> TriviallyNullable TriviallyNullable;
            RecursivelyNullable -> OnlyUsesNullableRules RecursivelyNullable;
            Literal -> "Literal";
            OneOf -> ["abcde"];
            NotNullable -> Literal TriviallyNullable OneOf;
        };
    }

    testcase! {
        nullability,
        &*NULLABILITY,
        &Grammar {
            rules: vec![
                rule!(TriviallyNullable -> ),
                rule!(OnlyUsesNullableRules -> TriviallyNullable TriviallyNullable),
                rule!(RecursivelyNullable -> OnlyUsesNullableRules RecursivelyNullable),
                rule!(Literal -> "Literal"),
                rule!(OneOf -> ["abcde"]),
                rule!(NotNullable -> Literal TriviallyNullable OneOf)
            ],
            nullables: hashset![
                String::from("TriviallyNullable"),
                String::from("OnlyUsesNullableRules"),
                String::from("RecursivelyNullable")
            ]
        }
    }

    tests! {
        rule_is_nullable:

        testcase! {
            trivially_nullable,
            NULLABILITY.rule_is_nullable("TriviallyNullable"),
            true
        }

        testcase! {
            nullable,
            NULLABILITY.rule_is_nullable("OnlyUsesNullableRules"),
            true
        }

        testcase! {
            recursively_nullable,
            NULLABILITY.rule_is_nullable("RecursivelyNullable"),
            true
        }

        testcase! {
            literal,
            NULLABILITY.rule_is_nullable("Literal"),
            false
        }

        testcase! {
            oneof,
            NULLABILITY.rule_is_nullable("OneOf"),
            false
        }

        testcase! {
            not_nullable,
            NULLABILITY.rule_is_nullable("NotNullable"),
            false
        }
    }
}
