use std::collections::HashSet;
use std::fmt;

/// Valid symbols for a [Rule](super::Rule) body
#[derive(Debug, PartialEq, Clone)]
pub enum Symbol {
    /// Succeeds if the [Rule](super::Rule) with the specified name succeeds
    Rule(String),
    /// Succeeds if the next character in the input matches the contained
    /// character
    Literal(char),
    /// Succeeds if the next character in the input matches any of the contained
    /// characters
    OneOf(HashSet<char>)
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::Rule(name) => write!(f, "{}", name),
            Symbol::Literal(text) => write!(f, "'{}'", text),
            Symbol::OneOf(chars) => {
                let mut chars = chars.into_iter().collect::<Vec<_>>();
                chars.sort_unstable();
                write!(f, "[{}]", chars.into_iter().collect::<String>())
            }
        }
    }
}

syntax_abuse::tests! {

    testcase! {
        rule,
        &symbol!(Rule)[0],
        &Symbol::Rule(String::from("Rule"))
    }

    testcase! {
        oneof,
        &symbol!(["12345"])[0],
        &Symbol::OneOf(hashset!['1', '2', '3', '4', '5'])
    }

    testcase! {
        single_literal,
        symbol!("1"),
        vec![Symbol::Literal('1')]
    }

    testcase! {
        several_literals,
        symbol!("12345"),
        vec![
            Symbol::Literal('1'),
            Symbol::Literal('2'),
            Symbol::Literal('3'),
            Symbol::Literal('4'),
            Symbol::Literal('5')
        ]
    }
}
