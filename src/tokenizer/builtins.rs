use super::{State, Tokenizer};

#[allow(unreachable_pub)]
pub use chain::chain;
#[allow(unreachable_pub)]
pub use eater::eat;
#[allow(unreachable_pub)]
pub use firstof::firstof;
#[allow(unreachable_pub)]
pub use literal::literal;
#[allow(unreachable_pub)]
pub use longestof::longestof;
#[allow(unreachable_pub)]
pub use map::map;
#[allow(unreachable_pub)]
pub use oneof::oneof;

mod chain;
mod eater;
mod firstof;
mod literal;
mod longestof;
mod map;
mod oneof;

/// Default token type for builtin tokenizers
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token type
    pub tag: &'static str,
    /// Characters covered by the token
    pub contents: String,
}

trait StateMachine {
    fn reset(&mut self) -> bool;
    fn feed(&mut self, c: char) -> State;
}

struct BasicTokenizer<S: StateMachine> {
    tag: &'static str,
    state: S,
}

impl<S: StateMachine> Tokenizer for BasicTokenizer<S> {
    type Token = Token;

    fn reset(&mut self) -> bool {
        self.state.reset()
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
