/// Valid symbols for a [Rule] body
#[derive(Debug, PartialEq)]
pub enum Symbol {
    /// Succeeds if the [Rule] with the specified name succeeds
    Rule(String),
    /// Succeeds if the next character in the input matches the contained
    /// character
    Literal(char),
    /// Succeeds if the next character in the input matches any of the contained
    /// characters
    OneOf(Vec<char>)
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
        &Symbol::OneOf(vec!['1', '2', '3', '4', '5'])
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
