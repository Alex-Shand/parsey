use super::{map::map, Tokenizer};

/// Run a sub-tokenizer but don't produce a token
///
/// Can also be used to coerce the token type of sub-tokenizers in aggregate
/// tokenizers such as `chain`. As `make_token` will never be called for
/// sub-tokenizers anyway it doesn't matter that `eat` doesn't ever produce a
/// token
pub fn eat<S, T: Tokenizer>(tokenizer: T) -> impl Tokenizer<Token = S> {
    map(tokenizer, |_| None)
}

syntax_abuse::tests! {
    use crate::tokenizer::{ tokenize, literal };

    testdata! {
        EATER: ??? = eat::<(), _>(literal("eaten", "test"));
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
