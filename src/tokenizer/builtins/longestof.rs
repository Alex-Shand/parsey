use super::{State, Tokenizer};

struct LongestOf<T> {
    tokenizers: Vec<Box<dyn Tokenizer<Token = T>>>,
    in_progress: Vec<usize>,
    last_completed: Option<usize>
}

impl<T> Tokenizer for LongestOf<T> {
    type Token = T;

    fn reset(&mut self) -> bool {
        self.in_progress = (0..self.tokenizers.len()).collect();
        self.last_completed = None;
        let mut can_complete_early = false;
        for tokenizer in &mut self.tokenizers {
            can_complete_early = tokenizer.reset();
        }
        can_complete_early
    }

    fn feed(&mut self, c: char) -> State {
        let mut completed = Vec::with_capacity(self.in_progress.len());
        let mut to_remove = Vec::with_capacity(self.in_progress.len());
        for (i, tokenizer_idx) in self.in_progress.iter().copied().enumerate() {
            let tokenizer = &mut self.tokenizers[tokenizer_idx];
            match tokenizer.feed(c) {
                State::Pending => (),
                State::Completed => completed.push(tokenizer_idx),
                State::Failed => to_remove.push(i),
            }
        }
        for i in to_remove.into_iter().rev() {
            let _ = self.in_progress.remove(i);
        }

        if !completed.is_empty() {
            self.last_completed = Some(completed[0]);
            State::Completed
        } else if self.in_progress.is_empty() {
            State::Failed
        } else {
            State::Pending
        }
    }

    fn make_token(&self, data: &[char]) -> Option<Self::Token> {
        self.tokenizers[self.last_completed.unwrap()].make_token(data)
    }
}

#[doc(hidden)]
#[must_use]
pub fn longestof<T>(tokenizers: Vec<Box<dyn Tokenizer<Token = T>>>) -> impl Tokenizer<Token = T> {
    let count = tokenizers.len();
    LongestOf {
        tokenizers,
        in_progress: (0..count).collect(),
        last_completed: None
    }
}

syntax_abuse::tests! {
    use crate::tokenizer::{literal, tokenize, Span, Token, TokenAndSpan};

    tests! {
        successes:

        testcase! {
            last,
            tokenize(
                "This is a test",
                longestof!(
                    literal("1", "This"),
                    literal("2", "This is"),
                    literal("3", "This is a"),
                    literal("4", "This is a test")
                )
            ),
            Ok(
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "4",
                            contents: String::from("This is a test")
                        },
                        span: Span::new(0, 0, 0, 13)
                    }
                ]
            )
        }

        testcase! {
            first,
            tokenize(
                "This is a test",
                longestof!(
                    literal("1", "This is a test"),
                    literal("2", "This is a"),
                    literal("3", "This is"),
                    literal("4", "This")
                )
            ),
            Ok(
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "1",
                            contents: String::from("This is a test")
                        },
                        span: Span::new(0, 0, 0, 13)
                    }
                ]
            )
        }

        testcase! {
            in_the_middle,
            tokenize(
                "This is a test",
                longestof!(
                    literal("1", "This is a"),
                    literal("2", "This is a test"),
                    literal("3", "This is"),
                    literal("4", "This")
                )
            ),
            Ok(
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "2",
                            contents: String::from("This is a test")
                        },
                        span: Span::new(0, 0, 0, 13)
                    }
                ]
            )
        }

        testcase! {
            tie,
            tokenize(
                "abcd",
                longestof!(
                    literal("1", "abcd"),
                    literal("2", "ab"),
                    literal("3", "abcd")
                )
            ),
            Ok(
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "1",
                            contents: String::from("abcd")
                        },
                        span: Span::new(0, 0, 0, 3)
                    }
                ]
            )
        }

        testcase! {
            longest_match_is_wrong,
            tokenize(
                "abcdabcd",
                longestof!(
                    literal("1", "ab"),
                    literal("2", "abcd"),
                    literal("3", "abcdef")
                )
            ),
            Ok(
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "2",
                            contents: String::from("abcd")
                        },
                        span: Span::new(0, 0, 0, 3)
                    },
                    TokenAndSpan {
                        token: Token {
                            tag: "2",
                            contents: String::from("abcd")
                        },
                        span: Span::new(0, 0, 4, 7)
                    }
                ]
            )
        }
    }

    tests! {
        failures:

        testcase! {
            all_too_short1,
            tokenize(
                "abcd",
                longestof!(
                    literal("1", "abc"),
                    literal("2", "ab"),
                    literal("3", "a")
                )
            ),
            Err((
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "1",
                            contents: String::from("abc")
                        },
                        span: Span::new(0, 0, 0, 2)
                    }
                ],
                String::from("d")
            ))
        }

        testcase! {
            all_too_short2,
            tokenize(
                "abcd",
                longestof!(
                    literal("1", "a"),
                    literal("2", "ab"),
                    literal("3", "abc")
                )
            ),
            Err((
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "3",
                            contents: String::from("abc")
                        },
                        span: Span::new(0, 0, 0, 2)
                    }
                ],
                String::from("d")
            ))
        }

        testcase! {
            all_too_long,
            tokenize(
                "abcd",
                longestof!(
                    literal("1", "abcde"),
                    literal("2", "abcdef"),
                    literal("3", "abcdefg")
                )
            ),
            Err((
                vec![],
                String::from("abcd")
            ))
        }

        testcase! {
            too_short_and_too_long,
            tokenize(
                "abcd",
                longestof!(
                    literal("1", "abc"),
                    literal("2", "abcde")
                )
            ),
            Err((
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "1",
                            contents: String::from("abc")
                        },
                        span: Span::new(0, 0, 0, 2)
                    }
                ],
                String::from("d")
            ))
        }

        testcase! {
            right_length_but_incorrect,
            tokenize(
                "abcd",
                longestof!(
                    literal("1", "abc"),
                    literal("2", "abc3"),
                    literal("3", "abcde")
                )
            ),
            Err((
                vec![
                    TokenAndSpan {
                        token: Token {
                            tag: "1",
                            contents: String::from("abc")
                        },
                        span: Span::new(0, 0, 0, 2)
                    }
                ],
                String::from("d")
            ))
        }
    }
}
