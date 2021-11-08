use super::{BasicTokenizer, State, StateMachine, Token, Tokenizer};

struct Chain<T> {
    tokenizers: Vec<Box<dyn Tokenizer<Token = T>>>,
    failed: bool,
    progress: usize,
}

impl<T> Chain<T> {
    fn the_rest_are_empty(&mut self) -> bool {
        self.tokenizers[self.progress..].iter().all(Tokenizer::can_match_empty)
    }
}

impl<T> StateMachine for Chain<T> {
    fn reset(&mut self) {
        self.failed = false;
        self.progress = 0;
        for tokenizer in &mut self.tokenizers {
            tokenizer.reset();
        }
    }

    fn can_match_empty(&self) -> bool {
        self.tokenizers.iter().all(|t| t.can_match_empty())
    }

    fn feed(&mut self, c: char) -> State {
        if self.failed || self.progress == self.tokenizers.len() {
            return State::Failed;
        }

        // The loop allows the match arms to jump back to the top if the match
        // by not returning
        loop {
            match self.tokenizers[self.progress].feed(c) {
                State::Pending => return State::Pending,
                State::Failed => {
                    // The current tokenizer can't accept the current character
                    // but if it can match the empty string we can skip it
                    if self.tokenizers[self.progress].can_match_empty() {
                        self.progress += 1;
                    } else {
                        return State::Failed;
                    }
                }
                State::Completed => {
                    //TODO: Need to implement the Completed, Failed sequence
                    // here before advancing progress
                    self.progress += 1;

                    // If the last tokenizer just completed or the remaining
                    // tokenizers can match the empty string then complete. The
                    // main tokenizer loop will still feed more characters if
                    // there are any
                    return if self.progress == self.tokenizers.len() || self.the_rest_are_empty() {
                        State::Completed
                    } else {
                        State::Pending
                    };
                }
            }
        }
    }
}

/// Implementation of the chain! macro
#[doc(hidden)]
#[must_use]
pub fn chain<T>(
    tag: &'static str,
    tokenizers: Vec<Box<dyn Tokenizer<Token = T>>>,
) -> impl Tokenizer<Token = Token> {
    BasicTokenizer {
        tag,
        state: Chain {
            tokenizers,
            progress: 0,
            failed: false,
        },
    }
}

syntax_abuse::tests! {
    use crate::tokenizer::{tokenize, literal, TokenAndSpan, Span};

    testcase! {
        simple,
        tokenize(
            "ABC",
            chain!(
                "chain",
                literal("", "A"),
                literal("", "B"),
                literal("", "C")
            )
        ),
        Ok(
            vec![
                TokenAndSpan {
                    token: Token {
                        tag: "chain",
                        contents: String::from("ABC")
                    },
                    span: Span::new(0, 0, 0, 3)
                }
            ]
        )
    }

    tests! {
        empty:

        testcase! {
            all,
            tokenize("", chain!("chain", literal("", ""), literal("", ""))),
            Ok(vec![])
        }

        testcase! {
            front,
            tokenize("AB", chain!("chain", literal("", ""), literal("", "A"), literal("", "B"))),
            Ok(
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "chain",
                            contents: String::from("AB")
                        },
                        span: Span::new(0, 0, 0, 2)
                    }
                ]
            )
        }

        testcase! {
            middle,
            tokenize("AB", chain!("chain", literal("", "A"), literal("", ""), literal("", "B"))),
            Ok(
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "chain",
                            contents: String::from("AB")
                        },
                        span: Span::new(0, 0, 0, 2)
                    }
                ]
            )
        }

        testcase! {
            end,
            tokenize("AB", chain!("chain", literal("", "A"), literal("", "B"), literal("", ""))),
            Ok(
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "chain",
                            contents: String::from("AB")
                        },
                        span: Span::new(0, 0, 0, 2)
                    }
                ]
            )
        }
    }

    testcase! {
        extra,
        tokenize(
            "ABCD",
            chain!("chain", literal("", "A"), literal("", "B"))
        ),
        Err((
            vec![
                TokenAndSpan {
                    token: Token {
                        tag: "chain",
                        contents: String::from("AB")
                    },
                    span: Span::new(0, 0, 0, 2)
                }
            ],
            String::from("CD")
        ))
    }

    tests! {
        failure:

        testcase! {
            in_the_first_tokenizer,
            tokenize(
                "Test123",
                chain!(
                    "chain",
                    literal("", "Text"),
                    literal("", "123")
                )
            ),
            Err((
                vec![],
                String::from("Test123")
            ))
        }

        testcase! {
            in_the_second_tokenizer,
            tokenize(
                "Test123",
                chain!(
                    "chain",
                    literal("", "Test"),
                    literal("", "13")
                )
            ),
            Err((
                vec![],
                String::from("Test123")
            ))
        }
    }
}
