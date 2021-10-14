use super::{ Tokenizer, State };

#[allow(unreachable_pub)]
pub use literal::literal;
#[allow(unreachable_pub)]
pub use oneof::oneof;
#[allow(unreachable_pub)]
pub use eater::eat;

mod literal;
mod oneof;
mod eater;

/// Default token type for builtin tokenizers
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token type
    pub tag: &'static str,
    /// Characters covered by the token
    pub contents: String,
}

trait StateMachine {
    fn reset(&mut self);
    fn feed(&mut self, c: char) -> State;
}

struct BasicTokenizer<S: StateMachine> {
    tag: &'static str,
    state: S,
}

impl<S: StateMachine> Tokenizer for BasicTokenizer<S> {
    type Token = Token;

    fn reset(&mut self) {
        self.state.reset();
    }

    fn feed(&mut self, c: char) -> State {
        self.state.feed(c)
    }

    fn make_token(&self, data: &[char]) -> Option<Self::Token> {
        Some(Token {
            tag: self.tag,
            contents: data.iter().collect(),
        })
    }
}
