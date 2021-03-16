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
struct Rule;

fn main() {
    println!("Hello, world!");
}

syntax_abuse::tests! {
    #[test]
    #[should_panic]
    fn empty_rules() {
        Grammar::new(Vec::new());
    }

    testcase! {
        non_empty_rules,
        Grammar::new(vec![Rule]),
        Grammar(vec![Rule])
    }
}
