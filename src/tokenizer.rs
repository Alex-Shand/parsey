//! Tokenizer definitions

use std::collections::HashSet;
use crate::utils::NonEmptyHashSet;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Span {
    start_line: usize,
    end_line: usize,
    start_char: usize,
    end_char: usize,
}

#[cfg(test)]
impl Span {
    fn new(start_line: usize, end_line: usize, start_char: usize, end_char: usize) -> Self {
        Span {
            start_line,
            end_line,
            start_char,
            end_char,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenAndSpan<T> {
    token: T,
    span: Span,
}

#[derive(Debug, Copy, Clone)]
enum TokenizerState {
    Pending,
    Completed,
    Failed,
}

trait Tokenizer {
    type Token;

    fn reset(&mut self);
    fn feed(&mut self, c: char) -> TokenizerState;
    fn make_token(&self, data: &[char]) -> Option<Self::Token>;

    fn complete(&mut self, data: &[char], span: Span) -> Option<TokenAndSpan<Self::Token>> {
        let token = self.make_token(data);
        self.reset();
        Some(TokenAndSpan { token: token?, span })
    }
}

fn tokenize<T, S: AsRef<str>>(
    input: S,
    mut tokenizer: impl Tokenizer<Token = T>,
) -> Result<Vec<TokenAndSpan<T>>, (Vec<TokenAndSpan<T>>, String)> {
    fn advance(c: char, end_line: &mut usize, end_char: &mut usize) {
        if c == '\n' {
            *end_line += 1;
            *end_char = 0;
        } else {
            *end_char += 1;
        }
    }

    fn make_error<T>(result: Vec<TokenAndSpan<T>>, chars: &[char], token_start: usize) -> Result<Vec<TokenAndSpan<T>>, (Vec<TokenAndSpan<T>>, String)> {
        Err((result, chars[token_start..].iter().collect()))
    }

    let mut chars = input.as_ref().chars().collect::<Vec<_>>();
    tokenizer.reset();

    let mut progress = 0;
    let mut token_start = 0;

    let mut start_line = 0;
    let mut end_line = 0;
    let mut start_char = 0;
    let mut end_char = 0;

    let mut result: Vec<TokenAndSpan<T>> = Vec::new();
    let mut last_result = TokenizerState::Pending;
    while progress != chars.len() {
        let c = chars[progress];
        last_result = tokenizer.feed(c);
        match last_result {
            TokenizerState::Pending => advance(c, &mut end_line, &mut end_char),
            TokenizerState::Completed => {
                if let Some(token) = tokenizer.complete(
                    &chars[token_start..progress + 1],
                    Span {
                        start_line,
                        end_line,
                        start_char,
                        end_char,
                    },
                ) {
                    result.push(token)
                }

                advance(c, &mut end_line, &mut end_char);
                token_start = progress + 1;
                start_line = end_line;
                start_char = end_char;
            }
            TokenizerState::Failed => return make_error(result, &chars, token_start)
        }
        progress += 1;
    }

    match last_result {
        TokenizerState::Completed => Ok(result),
        TokenizerState::Pending => make_error(result, &chars, token_start),
        TokenizerState::Failed => unreachable!()
    }
}

trait StateMachine {
    fn reset(&mut self);
    fn feed(&mut self, c: char) -> TokenizerState;
}

impl StateMachine for () {
    fn reset(&mut self) {
        panic!("Stub");
    }

    fn feed(&mut self, c: char) -> TokenizerState {
        panic!("Stub");
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Token {
    tag: &'static str,
    contents: String,
}

struct BasicTokenizer<S: StateMachine> {
    tag: &'static str,
    state: S,
}

impl<S: StateMachine> Tokenizer for BasicTokenizer<S> {
    type Token = Token;

    fn reset(&mut self) {
        self.state.reset()
    }

    fn feed(&mut self, c: char) -> TokenizerState {
        self.state.feed(c)
    }

    fn make_token(&self, data: &[char]) -> Option<Self::Token> {
        Some(Token {
            tag: self.tag,
            contents: data.iter().collect(),
        })
    }
}

struct Literal {
    progress: usize,
    data: Vec<char>,
}

impl Literal {
    fn new<S: AsRef<str>>(lit: S) -> Self {
        Literal {
            progress: 0,
            data: lit.as_ref().chars().collect(),
        }
    }
}

impl StateMachine for Literal {
    fn reset(&mut self) {
        self.progress = 0;
    }

    fn feed(&mut self, c: char) -> TokenizerState {
        if self.progress == self.data.len() {
            return TokenizerState::Failed;
        }
        if c != self.data[self.progress] {
            return TokenizerState::Failed;
        }
        self.progress += 1;
        if self.progress == self.data.len() {
            TokenizerState::Completed
        } else {
            TokenizerState::Pending
        }
    }
}

fn literal<S: AsRef<str>>(tag: &'static str, lit: S) -> impl Tokenizer<Token = Token> {
    BasicTokenizer {
        tag,
        state: Literal::new(lit),
    }
}

struct OneOf {
    chars: NonEmptyHashSet<char>,
    done: bool
}

impl OneOf {
    fn new(chars: HashSet<char>) -> Self {
        OneOf { chars: NonEmptyHashSet::new(chars), done: false }
    }
}

impl StateMachine for OneOf {
    fn reset(&mut self) {
        self.done = false;
    }

    fn feed(&mut self, c: char) -> TokenizerState {
        if self.done || !self.chars.contains(&c) {
            return TokenizerState::Failed;
        }
        self.done = true;
        TokenizerState::Completed
    }
}

fn oneof(tag: &'static str, chars: HashSet<char>) -> impl Tokenizer<Token = Token> {
    BasicTokenizer { tag, state: OneOf::new(chars) }
}

struct Eater<T: Tokenizer> {
    tokenizer: T
}

impl<T: Tokenizer> Tokenizer for Eater<T> {
    type Token = T::Token;

    fn reset(&mut self) {
        self.tokenizer.reset();
    }

    fn feed(&mut self, c: char) -> TokenizerState {
        self.tokenizer.feed(c)
    }

    fn make_token(&self, data: &[char]) -> Option<Self::Token> {
        None
    }
}

fn eat<T>(tokenizer: impl Tokenizer<Token = T>) -> impl Tokenizer<Token = T> {
    Eater { tokenizer }
}

// fn repeated<T, D>(token: impl Tokenizer<Token = T>, delimeter: Option<impl Tokenizer<Token = D>>, min: usize, max: usize) -> impl Tokenizer<Token = T> {
//     todo!()
// }

// fn firstof<T>(tokenizers: Vec<Box<dyn Tokenizer<Token = T>>>) -> impl Tokenizer<Token = T> {
//     todo!()
// }

// fn longestof<T>(tokenizers: Vec<Box<dyn Tokenizer<Token = T>>>) ->impl Tokenizer<Token = T> {
//     todo!()
// }

syntax_abuse::tests! {
    tests! {
        literal:

        testcase! {
            simple,
            tokenize(
                "test",
                literal("simple", "test")
            ),
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
            extra,
            tokenize(
                "Text More Text",
                literal("extra", "Text")
            ),
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
            tokenize(
                "1234",
                literal("not-enough", "12345"),
            ),
            Err((
                vec![],
                String::from("1234")
            ))
        }

        testcase! {
            failure,
            tokenize(
                "Text",
                literal("failure", "Test")
            ),
            Err((
                vec![],
                String::from("Text")
            ))
        }
    }

    tests! {
        oneof:

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
                        span: Span::new(0, 0, 0, 0)
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
                        span: Span::new(0, 0, 0, 0)
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
                        span: Span::new(0, 0, 0, 0)
                    },
                    TokenAndSpan {
                        token: Token {
                            tag: "simple",
                            contents: String::from("B")
                        },
                        span: Span::new(0, 0, 1, 1)
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

    tests! {
        eat:

        testdata! {
            EATER: ??? = eat(literal("eaten", "test"));
        }

        testcase! {
            simple,
            tokenize("test", EATER!()),
            Ok(vec![])
        }

        testcase! {
            extra,
            tokenize("test extra", EATER!()),
            Err((
                vec![],
                String::from(" extra")
            ))
        }

        testcase! {
            failure,
            tokenize("text", EATER!()),
            Err((
                vec![],
                String::from("text")
            ))
        }
    }
}
