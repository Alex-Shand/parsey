use std::iter::FromIterator;

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

    pub fn recognise<S: AsRef<str>>(&self, input: S) -> bool {
        let input = input.as_ref().chars().collect::<Vec<_>>();
        let mut parse_state = vec![
            self.get_rules_by_name(self.start_symbol()).iter()
                .map(|rule| rule.to_earley_item(State { start: 0, progress: 0}))
                .collect::<StateSet>()
        ];
        let mut current_position = 0;
        loop {
            if current_position >= input.len() {
                break;
            }
            
            if let Some(current_state) = parse_state.get_mut(current_position) {
                while let Some(item) = current_state.next() {
                    match item.parse(&self) {
                        ParseResult::Predict(rules) => {
                            current_state.add(
                                rules.iter()
                                    .map(|rule|
                                         rule.to_earley_item(
                                             State {
                                                 start: current_position,
                                                 progress: 0
                                             }
                                         )
                                    )
                            )
                        }
                    }
                }
            } else {
                break;
            }

            current_position += 1;
        }
        todo!("Did the parse work?")
    }

    fn start_symbol(&self) -> &str {
        &self.0[0].name
    }

    fn get_rules_by_name(&self, name: &str) -> Vec<&Rule> {
        self.0.iter().filter(|rule| rule.name == name).collect::<Vec<_>>()
    }
}

struct StateSet<'a> {
    items: Vec<Item<'a>>,
    next: usize
}

impl<'a> StateSet<'a> {
    fn next(&mut self) -> Option<&Item<'a>> {
        let current = self.next;
        self.next += 1;
        self.items.get(current)
    }

    fn add<I>(&mut self, new_items: I) where I: Iterator<Item=Item<'a>> {
        for item in new_items {
            if !self.items.contains(&item) {
                self.items.push(item)
            }
        }
    }
}

impl<'a> FromIterator<Item<'a>> for StateSet<'a> {
    fn from_iter<T>(it: T) -> Self
    where T: IntoIterator<Item=Item<'a>> {
        StateSet {
            items: it.into_iter().collect::<Vec<Item<'a>>>(),
            next: 0
        }
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

    fn to_earley_item(&self, state: State) -> Item<'_> {
        assert!(
            state.progress <= self.body.len(),
            "Progress is {} but the rule only has {} items",
            state.progress,
            self.body.len()
        );
        Item { rule: &self, state: state }
    }
}

#[derive(Debug, PartialEq)]
struct Item<'a> {
    rule: &'a Rule,
    state: State
}

#[derive(Debug)]
enum ParseResult<'a> {
    Predict(Vec<&'a Rule>)
}

impl Item<'_> {
    fn parse<'a>(&self, grammar: &'a Grammar) -> ParseResult<'a> {
        if let Some(matcher) = self.rule.body.get(self.state.progress) {
            match matcher {
                Matcher::Rule(name) =>
                    ParseResult::Predict(grammar.get_rules_by_name(name)),
                _ => todo!("Scan")
            }
        } else {
            todo!("Completion")
        }
    }
}

#[derive(Debug, PartialEq)]
struct State {
    start: usize,
    progress: usize
}

/// Valid symbols for a [Rule] body
#[derive(Debug, PartialEq)]
pub enum Matcher {
    /// Succeeds if the [Rule] with the specified name succeeds
    Rule(String),
    /// Succeeds if the next character in the input matches the contained
    /// character
    Literal(char),
    /// Succeeds if the next character in the input matches any of the contained
    /// characters
    OneOf(Vec<char>)
}

/// Parses matcher syntax on behalf of rule!()
///
/// Note: All branches return a Vec to enable uniform treatment inside rule! and
/// because the Literal branch can expand to multiple Literals
#[macro_export]
#[doc(hidden)]
macro_rules! matcher {
    // Rule syntax is a bareword (ident doesn't cover all legal possibilities
    // but does cover all sensible ones). The output matcher contains the
    // stringified rule name.
    ($rule:ident) => {
        vec![
            $crate::grammar::Matcher::Rule(
                String::from(::std::stringify!($rule))
            )
        ]
    };
    // OneOf is a string literal surrounded by [] (which is conveniently similar
    // to a regex character class and makes the whole thing one token tree in
    // rule!)
    ([ $str:literal ]) => {
        vec![$crate::grammar::Matcher::OneOf($str.chars().collect::<Vec<_>>())]
    };
    // A string literal without [] is a sequence of Literal matchers (one for
    // each character in the string)
    ($str:literal) => {
        $str.chars().map($crate::grammar::Matcher::Literal).collect::<Vec<_>>()
    };
}

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

/// Helper for grammar! { }. Collects rules by finding each ; then passing the
/// preceding token trees to rule!().
#[macro_export]
#[doc(hidden)]
macro_rules! grammar_aux {
    // Base case: Found all of the rules and don't have any leftover tokens,
    // construct a new grammar.
    ([][$($rules:expr)*]) => {
        $crate::grammar::Grammar::new(vec![$($rules),*])
    };
    // No more tokens in the input but there are still some in the
    // accumulator. Assume that they represent a rule (this is caused by missing
    // the ; from the last rule).
    ([$($rule:tt)+][$($rules:expr)*]) => {
        $crate::grammar_aux!([][$($rules)* $crate::rule!($($rule)*)]);
    };
    // Found a ;. Assume everything preceding it (now in the accumulator) is one
    // rule. The rule is constructed with rule! then pushed onto the rules list
    ([$($rule:tt)*][$($rules:expr)*] ; $($rest:tt)*) => {
        $crate::grammar_aux!([][$($rules)* $crate::rule!($($rule)*)] $($rest)*)
    };
    // Something other than a ;. Push it onto the accumulator then recuse on the
    // remaining input.
    ([$($acc:tt)*][$($rules:expr)*] $first:tt $($rest:tt)*) => {
        $crate::grammar_aux!([$($acc)* $first][$($rules)*] $($rest)*)
    };
}

/// Macro to construct a [Grammar]. Expands to appropriate calls to
/// [Grammar::new] and [Rule::new]
///
/// # Syntax
/// ```
/// grammar! {
///     <Rule Name> -> <Rule Body>;
///     ...
/// }
/// ```
/// Within the rule body an unquoted rule name becomes [Matcher::Rule]
/// ```
/// grammar! {
///     // Rule::new("Rule", vec![Matcher::Rule("AnotherRule")])
///     Rule -> AnotherRule;
/// }
/// ```
/// A bare string becomes a sequence of [Matcher::Literal] (one for each character)
/// ```
/// grammar! {
///     // Rule::new("Rule", vec![Matcher::Literal('1'), Matcher::Literal('2')])
///     Rule -> "12";
/// }
/// ```
/// A string wrapped in `[]` is [Matcher::OneOf]
/// ```
/// grammar! {
///     // Rule::new("Rule", vec![Matcher::OneOf(vec!['1','2'])])
///     Rule -> ["12"];
/// }
/// ```
///
/// # Panics
/// See [Grammar::new] and [Rule::new]
///
/// # Examples
/// ```
/// grammar! {
///     Sum -> Sum ["+-"] Product;
///     Sum -> Product;
///     Product -> Product ["*/"] Factor;
///     Product -> Factor;
///     Factor -> "(" Sum ")";
///     Factor -> Number;
///     Number -> ["0123456789"] Number;
///     Number -> ["0123456789"];
/// }
/// ```
#[macro_export]
macro_rules! grammar {
    ($($rules:tt)+) => {
        // Construct the new grammar with grammar_aux initialised with two empty
        // accumulators
        $crate::grammar_aux!([][] $($rules)+)
    };
}

syntax_abuse::tests! {

    tests! {
        recogniser:

        macro_rules! tc {
            ($name:ident, $input:expr, $expected:expr) => {
                testcase! {
                    $name,
                    grammar! {
                        Sum -> Sum ["+-"] Product;
                        Sum -> Product;
                        Product -> Product ["*/"] Factor;
                        Product -> Factor;
                        Factor -> "(" Sum ")";
                        Factor -> Number;
                        Number -> ["0123456789"] Number;
                        Number -> ["0123456789"];
                    }.recognise($input),
                    $expected
                }
            }
        }

        tc! {
            success,
            "1+2+3-4+5*(6+7)/106",
            true
        }

        tc! {
            truncated_input,
            "1+",
            false
        }

        tc! {
            invalid_character,
            "1%2",
            false
        }
    }

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
