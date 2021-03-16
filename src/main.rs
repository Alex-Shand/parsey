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
struct Matcher;

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
            Rule::new(String::from("Rule"), vec![Matcher]),
            Rule { name: String::from("Rule"), body: vec![Matcher] }
        }
    }
}
