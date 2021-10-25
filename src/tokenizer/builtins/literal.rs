use super::{BasicTokenizer, State, StateMachine, Token, Tokenizer};

struct Literal {
    progress: usize,
    data: Vec<char>,
}

impl StateMachine for Literal {
    fn reset(&mut self) -> bool {
        self.progress = 0;
        self.data.is_empty()
    }

    fn feed(&mut self, c: char) -> State {
        if self.progress == self.data.len() {
            return State::Failed;
        }
        if c != self.data[self.progress] {
            return State::Failed;
        }
        self.progress += 1;
        if self.progress == self.data.len() {
            State::Completed
        } else {
            State::Pending
        }
    }
}

/// Match a literal sequence of characters
pub fn literal<S: AsRef<str>>(tag: &'static str, lit: S) -> impl Tokenizer<Token = Token> {
    BasicTokenizer {
        tag,
        state: Literal {
            progress: 0,
            data: lit.as_ref().chars().collect(),
        },
    }
}

syntax_abuse::tests! {
    use crate::tokenizer::{ tokenize, TokenAndSpan, Span };

    testcase! {
        simple,
        tokenize("test", literal("simple", "test")),
        Ok(
            vec![
                TokenAndSpan {
                    token: Token {
                        tag: "simple",
                        contents: String::from("test")
                    },
                    span: Span::new(0, 0, 0, 3)
                }
            ]
        )
    }

    testcase! {
        newline,
        tokenize(
            "First Line\nSecond Line",
            literal("newline", "First Line\nSecond Line")
        ),
        Ok(
            vec![
                TokenAndSpan {
                    token: Token {
                        tag: "newline",
                        contents: String::from("First Line\nSecond Line")
                    },
                    span: Span::new(0, 1, 0, 10)
                }
            ]
        )
    }

    testcase! {
        empty,
        tokenize("", literal("empty", "")),
        Ok(vec![])
    }

    testcase! {
        extra,
        tokenize("Text More Text", literal("extra", "Text")),
        Err((
            vec![
                TokenAndSpan {
                    token: Token {
                        tag: "extra",
                        contents: String::from("Text")
                    },
                    span: Span::new(0, 0, 0, 3)
                }
            ],
            String::from(" More Text")
        ))
    }

    testcase! {
        not_enough,
        tokenize("1234", literal("not-enough", "12345")),
        Err((
            vec![],
            String::from("1234")
        ))
    }

    testcase! {
        failure,
        tokenize("Text", literal("failure", "Test")),
        Err((
            vec![],
            String::from("Text")
        ))
    }
}
