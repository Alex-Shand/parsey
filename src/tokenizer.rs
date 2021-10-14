//! Tokenizer

pub use span::Span;
pub use builtins::{
    Token,
    literal,
    oneof,
    eat
};

mod span;
mod builtins;

type Tokens<T> = Vec<TokenAndSpan<T>>;
type Result<T> = std::result::Result<Tokens<T>, (Tokens<T>, String)>;

/// The token and source span information
#[derive(Debug, Clone, PartialEq)]
pub struct TokenAndSpan<T> {
    /// The token
    pub token: T,
    /// The span
    pub span: Span,
}

/// Tokenization States
#[derive(Debug, Copy, Clone)]
pub enum State {
    /// Tokenizer is waiting for more input
    Pending,
    /// Tokenizer is ready to produce a token
    Completed,
    /// Tokenizer has failed
    Failed,
}

/// Trait for custom Tokenizers
pub trait Tokenizer {
    /// Token type
    type Token;

    /// Reset the tokenizer
    ///
    /// This will be called before tokenization starts and after each call to
    /// `make_token`
    fn reset(&mut self);

    /// Send a character to the tokenizer
    ///
    /// The tokenizer only needs to provide a state transition and doesn't have
    /// to store the character. Once the tokenizer completes it will be passed
    /// all of the characters again in `make_token` once `feed` returns
    /// `Completed`
    fn feed(&mut self, c: char) -> State;

    /// Allocate a token, will only be called once `feed` returns `Completed`
    ///
    /// May return `None` to avoid producing a token, in this case the input is
    /// still consumed
    fn make_token(&self, data: &[char]) -> Option<Self::Token>;
}

struct TokenizationState<T: Tokenizer> {
    tokenizer: T,
    chars: Vec<char>,
    progress: usize,
    token_start: usize,
    start_line: usize,
    end_line: usize,
    start_char: usize,
    end_char: usize,
    last_result: State,
}

impl<T: Tokenizer> TokenizationState<T> {
    fn tokenize(mut self) -> Result<T::Token> {
        self.tokenizer.reset();

        let mut result: Tokens<T::Token> = Vec::new();
        while self.progress != self.chars.len() {
            let c = self.chars[self.progress];
            self.last_result = self.tokenizer.feed(c);
            match self.last_result {
                State::Pending => self.advance(c),
                State::Completed => {
                    if let Some(token) = self.complete() {
                        result.push(token);
                    }

                    self.advance(c);
                    self.token_start = self.progress + 1;
                    self.start_line = self.end_line;
                    self.start_char = self.end_char;
                }
                State::Failed => return self.make_error(result),
            }
            self.progress += 1;
        }

        match self.last_result {
            State::Completed => Ok(result),
            State::Pending => self.make_error(result),
            State::Failed => unreachable!(),
        }
    }

    fn advance(&mut self, c: char) {
        if c == '\n' {
            self.end_line += 1;
            self.end_char = 0;
        } else {
            self.end_char += 1;
        }
    }

    fn make_error(self, result: Tokens<T::Token>) -> Result<T::Token> {
        Err((result, self.chars[self.token_start..].iter().collect()))
    }

    fn complete(&mut self) -> Option<TokenAndSpan<T::Token>> {
        let token = self.tokenizer.make_token(&self.chars[self.token_start..=self.progress]);
        self.tokenizer.reset();
        Some(TokenAndSpan {
            token: token?,
            span: Span::new(
                self.start_line,
                self.end_line,
                self.start_char,
                self.end_char,
            )
        })
    }
}

/// Tokenize a string
///
/// # Errors
/// If the tokenizer fails or consumes the whole input without completing it
/// returns all of the tokens found and the remaining unconsumed input if any
pub fn tokenize<T, S: AsRef<str>>(
    input: S,
    tokenizer: impl Tokenizer<Token = T>,
) -> Result<T> {
    TokenizationState {
        tokenizer,
        chars: input.as_ref().chars().collect(),
        progress: 0,
        token_start: 0,
        start_line: 0,
        end_line: 0,
        start_char: 0,
        end_char: 0,
        last_result: State::Pending
    }.tokenize()
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
