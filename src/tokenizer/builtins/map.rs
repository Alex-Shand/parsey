use super::{State, Tokenizer};

struct Mapper<S, T: Tokenizer, F: Fn(&[char]) -> Option<S>> {
    tokenizer: T,
    make_token: F,
}

impl<S, T: Tokenizer, F: Fn(&[char]) -> Option<S>> Tokenizer for Mapper<S, T, F> {
    type Token = S;

    fn reset(&mut self) -> bool {
        self.tokenizer.reset()
    }

    fn feed(&mut self, c: char) -> State {
        self.tokenizer.feed(c)
    }

    fn make_token(&self, data: &[char]) -> Option<Self::Token> {
        (self.make_token)(data)
    }
}

/// Override the `make_token` method of a tokenizer
///
/// The state machine defined by the sub-tokenizer's `reset` and `feed` methods
/// is still used to drive tokenization but the `make_token` argument is called
/// instead of the sub-tokenizer's `make_token`
pub fn map<S, T, F: Fn(&[char]) -> Option<T>>(
    tokenizer: impl Tokenizer<Token = S>,
    make_token: F,
) -> impl Tokenizer<Token = T> {
    Mapper {
        tokenizer,
        make_token,
    }
}

syntax_abuse::tests! {
    use crate::tokenizer::{ tokenize, literal, TokenAndSpan, Span };

    testdata! {
        MAPPER: ??? = map(literal("map", "test"), |chars| Some(chars.iter().collect::<String>()));
    }

    testcase! {
        simple,
        tokenize("test", MAPPER!()),
        Ok(
            vec![
                TokenAndSpan {
                    token: String::from("test"),
                    span: Span::new(0, 0, 0, 4)
                }
            ]
        )
    }

    testcase! {
        extra,
        tokenize("test extra", MAPPER!()),
        Err((
            vec![
                TokenAndSpan {
                    token: String::from("test"),
                    span: Span::new(0, 0, 0, 4)
                }
            ],
            String::from(" extra")
        ))
    }

    testcase! {
        failure,
        tokenize("text", MAPPER!()),
        Err((
            vec![],
            String::from("text")
        ))
    }
}
