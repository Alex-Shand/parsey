//! Tokenizer
use std::rc::Rc;
use std::cell::RefCell;

pub use builtins::{chain, eat, firstof, literal, longestof, map, oneof, Token};
pub use span::{ Span, CharacterPosition };

mod builtins;
mod span;

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
    fn reset(&mut self) -> bool;

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

/// Persistent tokenization state
struct TokenizationState<T: Tokenizer> {
    tokenizer: Rc<RefCell<T>>,
    chars: Rc<Vec<char>>,
    progress: usize,
    token_start: usize,
    start_line: usize,
    end_line: usize,
    start_char: usize,
    end_char: usize,
    last_result: State
}

impl<T: Tokenizer> Clone for TokenizationState<T> {
    fn clone(&self) -> Self {
        TokenizationState {
            tokenizer: self.tokenizer.clone(),
            chars: self.chars.clone(),
            progress: self.progress,
            token_start: self.token_start,
            start_line: self.start_line,
            end_line: self.end_line,
            start_char: self.start_char,
            end_char: self.end_char,
            last_result: self.last_result
        }
    }
}

impl<T: Tokenizer> TokenizationState<T> {
    fn tokenize(mut self) -> Result<T::Token> {
        // The tokens found so far
        let mut result = Vec::new();
        // A copy of the state from the last time the tokenizer completed
        let mut candidate = None;

        while !self.eof() {
            self.last_result = self.tokenizer.borrow_mut().feed(self.chars[self.progress]);
            match self.last_result {
                // Nothing to do until the tokenizer yields something or fails
                State::Pending => self.advance(),
                // The tokenizer could produce a token at this position. Don't
                // actually produce a token yet (we need to check it is the
                // longest possible token) but save the state
                State::Completed => {
                    // Need to advance across the current character first
                    self.advance();
                    candidate = Some(self.clone());
                }
                // The tokenizer can't accept any more input. Work out if it
                // has a token
                State::Failed => {
                    if candidate.is_some() {
                        // The tokenizer completed at some point in the past (any
                        // number of Pendings can happen between the last Completed
                        // and now)

                        // Reset the state to the point the tokenizer last
                        // completed. Resets candidate to None in the process so
                        // the next Failed will take the other branch
                        self = candidate.take().unwrap();
                        // Add a token to result without moving the progress
                        // marker forward (the current character will be fed to
                        // the tokenizer again in the next loop iteration after
                        // it has been reset)
                        self.complete(&mut result);
                    } else {
                        // The tokenizer failed without ever completing, fail
                        // immediately
                        return self.make_error(result);
                    }
                }
            }
        }

        // If the tokenizer completes on the last character of the input or only
        // produces Pending after the last completion the loop above can exit
        // without using the candidate. If there is one still around restore to
        // it and produce a token
        if let Some(candidate) = candidate {
            self = candidate;
            self.complete(&mut result);
        }

        // Because of the candidate restore above we might not be at the end of
        // input anymore but we know there were no completions between the
        // current position and the end of input so anything left over is
        // unconsumed
        if !self.eof() {
            return self.make_error(result);
        }

        // If there were no completions we will reach this point with
        // last_result == Pending and want to produce an error. Failed is
        // impossible as the main loop either falls back to the last completion
        // or bails out when it encounters a failure.
        match self.last_result {
            State::Completed => Ok(result),
            State::Pending => self.make_error(result),
            State::Failed => unreachable!(),
        }
    }

    /// Update tokenization state based on the current character
    fn advance(&mut self) {
        if self.chars[self.progress] == '\n' {
            self.end_line += 1;
            self.end_char = 0;
        } else {
            self.end_char += 1;
        }
        self.progress += 1;
    }

    /// Produce the error result, errors contain all of the tokens found and any
    /// input left over
    fn make_error(self, result: Tokens<T::Token>) -> Result<T::Token> {
        Err((result, self.chars[self.token_start..].iter().collect()))
    }

    /// True of the tokenizer has reached end of input, false otherwise
    fn eof(&mut self) -> bool {
        self.progress == self.chars.len()
    }

    /// If the tokenizer produces a token add it to result then update
    /// tokenization state
    fn complete(&mut self, result: &mut Tokens<T::Token>) {
        // Tokenizers can return None from make token to consume the input but
        // not add a token to the result (e.g whitespace or comments)
        if let Some(token) = self
            .tokenizer.borrow()
            .make_token(&self.chars[self.token_start..self.progress]) {
                result.push(TokenAndSpan {
                    token,
                    span: Span::new(
                        self.start_line,
                        self.end_line,
                        self.start_char,
                        self.end_char,
                    )
                });
        }

        // Reset the tokenizer for the next token
        //TODO: It seems like the return value of reset should be important here
        let _ = self.tokenizer.borrow_mut().reset();

        // Update the variables tracking the beginning of the new token
        self.token_start = self.progress;
        self.start_line = self.end_line;
        self.start_char = self.end_char;
    }
}

/// Tokenize a string
///
/// # Errors
/// If the tokenizer fails or consumes the whole input without completing it
/// returns all of the tokens found and the remaining unconsumed input if any
pub fn tokenize<T, S: AsRef<str>>(input: S, mut tokenizer: impl Tokenizer<Token = T>) -> Result<T> {
    let already_completed = tokenizer.reset();
    TokenizationState {
        tokenizer: Rc::new(RefCell::new(tokenizer)),
        chars: Rc::new(input.as_ref().chars().collect()),
        progress: 0,
        token_start: 0,
        start_line: 0,
        end_line: 0,
        start_char: 0,
        end_char: 0,
        last_result: if already_completed { State::Completed } else { State::Pending }
    }
    .tokenize()
}

// fn repeated<T, D>(token: impl Tokenizer<Token = T>, delimeter: Option<impl Tokenizer<Token = D>>, min: usize, max: usize) -> impl Tokenizer<Token = T> {
//     todo!()
// }
