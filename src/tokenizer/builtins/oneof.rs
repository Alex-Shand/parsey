use super::{BasicTokenizer, State, StateMachine, Token, Tokenizer};
use crate::utils::NonEmptyHashSet;
use std::collections::HashSet;

struct OneOf {
    chars: NonEmptyHashSet<char>,
    done: bool,
}

impl StateMachine for OneOf {
    fn reset(&mut self) -> bool {
        self.done = false;
        false
    }

    fn feed(&mut self, c: char) -> State {
        if self.done || !self.chars.contains(&c) {
            return State::Failed;
        }
        self.done = true;
        State::Completed
    }
}

/// Match a single character from a set of characters
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn oneof(tag: &'static str, chars: HashSet<char>) -> impl Tokenizer<Token = Token> {
    BasicTokenizer {
        tag,
        state: OneOf {
            chars: NonEmptyHashSet::new(chars),
            done: false,
        },
    }
}

syntax_abuse::tests! {
    use crate::tokenizer::{ tokenize, TokenAndSpan, Span };

    testdata! {
        SIMPLE: ??? = oneof("simple", hashset!['A', 'B']);
    }

    testcase! {
        simple1,
        tokenize("A", SIMPLE!()),
        Ok(
            vec![
                TokenAndSpan {
                    token: Token {
                        tag: "simple",
                        contents: String::from("A")
                    },
                    span: Span::new(0, 0, 0, 1)
                }
            ]
        )
    }

    testcase! {
        simple2,
        tokenize("B", SIMPLE!()),
        Ok(
            vec![
                TokenAndSpan {
                    token: Token {
                        tag: "simple",
                        contents: String::from("B")
                    },
                    span: Span::new(0, 0, 0, 1)
                }
            ]
        )
    }

    testcase! {
        extra,
        tokenize("ABC", SIMPLE!()),
        Err((
            vec![
                TokenAndSpan {
                    token: Token {
                        tag: "simple",
                        contents: String::from("A")
                    },
                    span: Span::new(0, 0, 0, 1)
                },
                TokenAndSpan {
                    token: Token {
                        tag: "simple",
                        contents: String::from("B")
                    },
                    span: Span::new(0, 0, 1, 2)
                }
            ],
            String::from("C")
        ))
    }

    testcase! {
        failure,
        tokenize("C", SIMPLE!()),
        Err((
            vec![],
            String::from("C")
        ))
    }
}
