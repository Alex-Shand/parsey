use super::{ Tokenizer, State };

struct Eater<T: Tokenizer> {
    tokenizer: T,
}

impl<T: Tokenizer> Tokenizer for Eater<T> {
    type Token = T::Token;

    fn reset(&mut self) {
        self.tokenizer.reset();
    }

    fn feed(&mut self, c: char) -> State {
        self.tokenizer.feed(c)
    }

    fn make_token(&self, _data: &[char]) -> Option<Self::Token> {
        None
    }
}

/// Run a sub-tokenizer but don't produce a token
pub fn eat<T>(tokenizer: impl Tokenizer<Token = T>) -> impl Tokenizer<Token = T> {
    Eater { tokenizer }
}

syntax_abuse::tests! {
    use crate::tokenizer::{ tokenize, literal };

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
