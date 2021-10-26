use super::{BasicTokenizer, State, StateMachine, Token, Tokenizer};

struct Chain<T> {
    tokenizers: Vec<Box<dyn Tokenizer<Token = T>>>,
    failed: bool,
    progress: usize,
}

impl<T> StateMachine for Chain<T> {
    fn reset(&mut self) -> bool {
        self.failed = false;
        self.progress = 0;

        let mut all_complete_early = true;
        for tokenizer in &mut self.tokenizers {
            all_complete_early &= tokenizer.reset();
        }
        all_complete_early
    }

    fn feed(&mut self, c: char) -> State {
        if self.failed || self.progress == self.tokenizers.len() {
            return State::Failed;
        }
        match self.tokenizers[self.progress].feed(c) {
            State::Pending => State::Pending,
            State::Failed => {
                self.failed = true;
                State::Failed
            }
            //TODO: Should be greedy
            State::Completed => {
                self.progress += 1;
                if self.progress == self.tokenizers.len() {
                    State::Completed
                } else {
                    State::Pending
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
