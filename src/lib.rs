use grammar::Grammar;
use state::{ StateSet, Item, State, ParseResult };

pub mod grammar;

mod state;

/// Return `true` if the input string is in the language described by
/// `grammar`, false otherwise.
pub fn recognise<S>(grammar: &Grammar, input: S) -> bool where S: AsRef<str> {
    let input = input.as_ref().chars().collect::<Vec<_>>();
    let mut parse_state = vec![
        StateSet::new(
            Item::from_rules(
                grammar.get_rules_by_name(grammar.start_symbol()),
                State { start: 0, progress: 0 }
            )
        )
    ];

    for current_position in 0..input.len() {
        if let Some(current_state) = parse_state.get_mut(current_position) {
            while let Some(item) = current_state.next() {
                match item.parse(grammar) {
                    ParseResult::Predict(rules) => {
                        current_state.add(
                            Item::from_rules(
                                rules,
                                State { start: current_position, progress: 0 }
                            )
                        )
                    }
                }
            }
        } else {
            todo!("Ran out of state before running out of input, this should be an error");
        }
    }
    todo!("Did the parse work?")
}

syntax_abuse::tests! {

    macro_rules! tc {
        ($name:ident, $input:expr, $expected:expr) => {
            testcase! {
                $name,
                recognise(
                    &grammar! {
                        Sum -> Sum ["+-"] Product;
                        Sum -> Product;
                        Product -> Product ["*/"] Factor;
                        Product -> Factor;
                        Factor -> "(" Sum ")";
                        Factor -> Number;
                        Number -> ["0123456789"] Number;
                        Number -> ["0123456789"];
                    },
                    $input
                ),
                $expected
            }
        }
    }

    tc! {
        success,
        "1+2+3-4+5*(6+7)/106",
        true
    }

    tc! {
        truncated_input,
        "1+",
        false
    }

    tc! {
        invalid_character,
        "1%2",
        false
    }
}
