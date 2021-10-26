use super::{State, Tokenizer};

struct FirstOf<T> {
    chosen_tokenizer: Option<usize>,
    tokenizers: Vec<Box<dyn Tokenizer<Token = T>>>,
}

impl<T> Tokenizer for FirstOf<T> {
    type Token = T;

    fn reset(&mut self) -> bool {
        self.chosen_tokenizer = None;
        let mut can_complete_early = false;
        for tokenizer in &mut self.tokenizers {
            can_complete_early = tokenizer.reset();
        }
        can_complete_early
    }

    fn feed(&mut self, c: char) -> State {
        if let Some(i) = self.chosen_tokenizer {
            self.tokenizers[i].feed(c)
        } else {
            for (i, tokenizer) in self.tokenizers.iter_mut().enumerate() {
                match tokenizer.feed(c) {
                    State::Failed => (),
                    success => {
                        self.chosen_tokenizer = Some(i);
                        return success;
                    }
                }
            }
            State::Failed
        }
    }

    fn make_token(&self, data: &[char]) -> Option<Self::Token> {
        self.tokenizers[self.chosen_tokenizer.unwrap()].make_token(data)
    }
}

#[doc(hidden)]
#[must_use]
pub fn firstof<T>(tokenizers: Vec<Box<dyn Tokenizer<Token = T>>>) -> impl Tokenizer<Token = T> {
    FirstOf {
        chosen_tokenizer: None,
        tokenizers,
    }
}

syntax_abuse::tests! {
    use crate::tokenizer::{literal, tokenize, Span, Token, TokenAndSpan};

    testdata! {
        TEST_OR_ABC: ??? = firstof!(
            literal("Test", "Test"),
            literal("abc", "abc")
        );
    }

    tests! {
        succeeds_on:

        testcase! {
            the_first_tokenizer,
            tokenize("Test", TEST_OR_ABC!()),
            Ok(
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "Test",
                            contents: String::from("Test")
                        },
                        span: Span::new(0, 0, 0, 4)
                    }
                ]
            )
        }

        testcase! {
            the_second_tokenizer,
            tokenize("abc", TEST_OR_ABC!()),
            Ok(
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "abc",
                            contents: String::from("abc")
                        },
                        span: Span::new(0, 0, 0, 3)
                    }
                ]
            )
        }
    }

    testcase! {
        failure,
        tokenize("123", TEST_OR_ABC!()),
        Err((
            vec![],
            String::from("123")
        ))
    }

    testcase! {
        longest_match_is_second,
        tokenize(
            "This is a test",
            firstof!(literal("short", "This"), literal("long", "This is a test"))
        ),
        Err((
            vec![
                TokenAndSpan {
                    token: Token {
                        tag: "short",
                        contents: String::from("This")
                    },
                    span: Span::new(0, 0, 0, 4)
                }
            ],
            String::from(" is a test")
        ))
    }
}
