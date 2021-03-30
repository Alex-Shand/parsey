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
        let mut to_add = Vec::new();
        let mut to_complete = Vec::new();
        if let Some(current_state) = parse_state.get_mut(current_position) {
            while let Some(item) = current_state.next() {
                match item.parse(grammar, input[current_position]) {
                    ParseResult::Predict(rules) => current_state.add(
                        Item::from_rules(
                            rules,
                            State {
                                start: current_position,
                                progress: 0
                            }
                        )
                    ),
                    ParseResult::Scan(item) => match item {
                        Some(item) => to_add.push(item),
                        None => ()
                    },
                    ParseResult::Complete(rule) => to_complete.push(rule)
                }
            }

            for (rule, start) in to_complete {
                to_add.extend(
                    parse_state[start].items().iter()
                        .filter_map(|item| item.next_name().and_then(|name| {
                            if name == rule {
                                Some(item.advanced())
                            } else {
                                None
                            }
                        }))
                )
            }

            if !to_add.is_empty() {
                parse_state.push(StateSet::new(to_add));
            }
        } else {
            println!("Input: {}", input.iter().collect::<String>());
            for (i, state) in parse_state.iter().enumerate() {
                println!("State Set: {}", i);
                println!("{}", state);
                println!();
            }
            todo!("Ran out of state before running out of input, this should be an error");
        }
    }
    println!("Input: {}", input.iter().collect::<String>());
    for (i, state) in parse_state.iter().enumerate() {
        println!("State Set: {}", i);
        println!("{}", state);
        println!();
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
//        "1+(2*3-4)",
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
