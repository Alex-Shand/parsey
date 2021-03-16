#[derive(Debug, PartialEq)]
pub struct Grammar(Vec<Rule>);

impl Grammar {
    pub fn new(rules: Vec<Rule>) -> Self {
        if rules.is_empty() {
            panic!("A grammar must have at least one rule")
        }
        Grammar(rules)
    }
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    name: String,
    body: Vec<Matcher>
}

impl Rule {
    pub fn new(name: String, body: Vec<Matcher>) -> Self {
        if name.starts_with("@") {
            panic!("Rule names beginning with @ are reserved")
        }
        Rule { name, body }
    }
}

#[derive(Debug, PartialEq)]
pub enum Matcher {
    Rule(String),
    Literal(char),
    OneOf(Vec<char>)
}

#[macro_export]
#[doc(hidden)]
macro_rules! matcher {
    ($rule:ident) => {
        vec![
            $crate::grammar::Matcher::Rule(
                String::from(::std::stringify!($rule))
            )
        ]
    };
    ([ $str:expr ]) => {
        vec![$crate::grammar::Matcher::OneOf($str.chars().collect::<Vec<_>>())]
    };
    ($str:expr) => {
        $str.chars().map($crate::grammar::Matcher::Literal).collect::<Vec<_>>()
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! rule {
    ($name:ident -> $($matchers:tt)*) => {
        $crate::grammar::Rule::new(
            String::from(::std::stringify!($name)),
            vec![
                $($crate::matcher!($matchers)),*
            ].into_iter().flatten().collect::<Vec<_>>()
        )
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! grammar_aux {
    (@[][$($rules:expr)*]) => {
        $crate::grammar::Grammar::new(vec![$($rules),*])
    };
    (@[$($rule:tt)+][$($rules:expr)*]) => {
        $crate::grammar_aux!(@[][$($rules)* $crate::rule!($($rule)*)]);
    };
    (@[$($rule:tt)*][$($rules:expr)*] ; $($rest:tt)*) => {
        $crate::grammar_aux!(@[][$($rules)* $crate::rule!($($rule)*)] $($rest)*)
    };
    (@[$($acc:tt)*][$($rules:expr)*] $first:tt $($rest:tt)*) => {
        $crate::grammar_aux!(@[$($acc)* $first][$($rules)*] $($rest)*)
    };
}

#[macro_export]
macro_rules! grammar {
    ($($rules:tt)+) => {
        $crate::grammar_aux!(@[][] $($rules)+)
    };
}

syntax_abuse::tests! {

    tests! {
        grammar:

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
                Rule {
                    name: String::from("Rule"),
                    body: vec![Matcher::Rule(String::from("Rule2"))]
                },
                Rule {
                    name: String::from("Rule2"),
                    body: vec![
                        Matcher::Literal('l'),
                        Matcher::Literal('i'),
                        Matcher::Literal('t'),
                        Matcher::Literal('e'),
                        Matcher::Literal('r'),
                        Matcher::Literal('a'),
                        Matcher::Literal('l')
                    ]
                }
            ])
        }

        testcase! {
            trailing_semi_is_optional,
            grammar! {
                Rule -> Rule2;
                Rule2 -> "literal"
            },
            Grammar(vec![
                Rule {
                    name: String::from("Rule"),
                    body: vec![Matcher::Rule(String::from("Rule2"))]
                },
                Rule {
                    name: String::from("Rule2"),
                    body: vec![
                        Matcher::Literal('l'),
                        Matcher::Literal('i'),
                        Matcher::Literal('t'),
                        Matcher::Literal('e'),
                        Matcher::Literal('r'),
                        Matcher::Literal('a'),
                        Matcher::Literal('l')
                    ]
                }
            ])
        }

        testcase! {
            only_one_rule,
            grammar! {
                Rule -> "literal"
            },
            Grammar(vec![
                Rule {
                    name: String::from("Rule"),
                    body: vec![
                        Matcher::Literal('l'),
                        Matcher::Literal('i'),
                        Matcher::Literal('t'),
                        Matcher::Literal('e'),
                        Matcher::Literal('r'),
                        Matcher::Literal('a'),
                        Matcher::Literal('l')
                    ]
                }
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
                Rule {
                    name: String::from("Sum"),
                    body: vec![
                        Matcher::Rule(String::from("Sum")),
                        Matcher::OneOf(vec!['+', '-']),
                        Matcher::Rule(String::from("Product"))
                    ]
                },
                Rule {
                    name: String::from("Sum"),
                    body: vec![
                        Matcher::Rule(String::from("Product"))
                    ]
                },
                Rule {
                    name: String::from("Product"),
                    body: vec![
                        Matcher::Rule(String::from("Product")),
                        Matcher::OneOf(vec!['*', '/']),
                        Matcher::Rule(String::from("Factor"))
                    ]
                },
                Rule {
                    name: String::from("Product"),
                    body: vec![
                        Matcher::Rule(String::from("Factor"))
                    ]
                },
                Rule {
                    name: String::from("Factor"),
                    body: vec![
                        Matcher::Literal('('),
                        Matcher::Rule(String::from("Sum")),
                        Matcher::Literal(')')
                    ]
                },
                Rule {
                    name: String::from("Factor"),
                    body: vec![
                        Matcher::Rule(String::from("Number"))
                    ]
                },
                Rule {
                    name: String::from("Number"),
                    body: vec![
                        Matcher::OneOf(vec![
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
                        Matcher::Rule(String::from("Number"))
                    ]
                },
                Rule {
                    name: String::from("Number"),
                    body: vec![
                        Matcher::OneOf(vec![
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
                }
            ])
        }
    }

    tests! {
        rule:

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

    tests! {
        matcher:

        testcase! {
            rule,
            &matcher!(Rule)[0],
            &Matcher::Rule(String::from("Rule"))
        }

        testcase! {
            oneof,
            &matcher!(["12345"])[0],
            &Matcher::OneOf(vec!['1', '2', '3', '4', '5'])
        }

        testcase! {
            single_literal,
            matcher!("1"),
            vec![Matcher::Literal('1')]
        }

        testcase! {
            several_literals,
            matcher!("12345"),
            vec![
                Matcher::Literal('1'),
                Matcher::Literal('2'),
                Matcher::Literal('3'),
                Matcher::Literal('4'),
                Matcher::Literal('5')
            ]
        }
    }
}
