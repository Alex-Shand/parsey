use super::{BasicTokenizer, State, StateMachine, Token, Tokenizer};

struct Chain<T> {
    tokenizers: Vec<Box<dyn Tokenizer<Token = T>>>,
    failed: bool,
    progress: usize,
}

impl<T> Chain<T> {
    fn skip_empty(&mut self) -> bool {
        let mut skipped = false;
        while self.progress < self.tokenizers.len()
            && self.tokenizers[self.progress].can_match_empty()
        {
            self.progress += 1;
            skipped = true;
        }
        skipped
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
        let mut result = true;
        for tokenizer in &self.tokenizers {
            result &= tokenizer.can_match_empty();
        }
        result
    }

    fn feed(&mut self, c: char) -> State {
        if self.failed || self.progress == self.tokenizers.len() {
            return State::Failed;
        }

        loop {
            match self.tokenizers[self.progress].feed(c) {
                State::Pending => return State::Pending,
                State::Failed => {
                    if !self.skip_empty() {
                        self.failed = true;
                        return State::Failed;
                    }
                }
                //TODO: Should be greedy
                State::Completed => {
                    self.progress += 1;
                    let _ = self.skip_empty();
                    return if self.progress == self.tokenizers.len() {
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
