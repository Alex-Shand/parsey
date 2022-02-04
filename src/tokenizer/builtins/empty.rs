use super::{eat, literal, Tokenizer};

/// Tokenizer that only matches the empty string
#[must_use]
pub fn empty<T>(tag: &'static str) -> impl Tokenizer<Token = T> {
    eat(literal(tag, ""))
}

syntax_abuse::tests! {
    use crate::tokenizer::tokenize;
    testcase! {
        test,
        tokenize("", empty::<()>("empty")),
        Ok(vec![])
    }
}
