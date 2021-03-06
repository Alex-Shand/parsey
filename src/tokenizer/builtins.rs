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
#[allow(unreachable_pub)]
pub use empty::empty;

mod chain;
mod eater;
mod firstof;
mod literal;
mod longestof;
mod map;
mod oneof;
mod empty;

/// Default token type for builtin tokenizers
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token type
    pub tag: &'static str,
    /// Characters covered by the token
    pub contents: String,
}

/// Tokenizer trait without the `make_token` function which is the same for all
/// tokenizers based on `BasicTokenizer`
trait StateMachine {
    fn reset(&mut self);
    fn can_match_empty(&self) -> bool;
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

    fn can_match_empty(&self) -> bool {
        self.state.can_match_empty()
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
