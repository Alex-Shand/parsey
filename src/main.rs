// Sum     -> Sum     [+-] Product
// Sum     -> Product
// Product -> Product [*/] Factor
// Product -> Factor
// Factor  -> '(' Sum ')'
// Factor  -> Number
// Number  -> [0-9] Number
// Number  -> [0-9]

#[derive(Debug, PartialEq)]
struct Grammar(Vec<Rule>);

impl Grammar {
    fn new(rules: Vec<Rule>) -> Self {
        if rules.is_empty() {
            panic!("A grammar must have at least one rule")
        }
        Grammar(rules)
    }
}

#[derive(Debug, PartialEq)]
struct Rule {
    name: String,
    body: Vec<Matcher>
}

impl Rule {
    fn new(name: String, body: Vec<Matcher>) -> Self {
        if name.starts_with("@") {
            panic!("Rule names beginning with @ are reserved")
        }
        Rule { name, body }
    }
}

#[derive(Debug, PartialEq)]
enum Matcher {
    Rule(String),
    Literal(char),
    OneOf(Vec<char>)
}

macro_rules! matcher {
    ($rule:ident) => {
        vec![Matcher::Rule(String::from(stringify!($rule)))]
    };
    ([ $str:expr ]) => {
        vec![Matcher::OneOf($str.chars().collect::<Vec<_>>())]
    };
    ($str:expr) => {
        $str.chars().map(Matcher::Literal).collect::<Vec<_>>()
    };
}

macro_rules! rule {
    ($name:ident -> $($matchers:tt),*) => {
        Rule::new(
            String::from(stringify!($name)),
            vec![
                $(matcher!($matchers)),*
            ].into_iter().flatten().collect::<Vec<_>>()
        )
    }
}

fn main() {
    println!("Hello, world!");
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
            rule!(Rule -> "literal", ["oneof"], Rule),
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
