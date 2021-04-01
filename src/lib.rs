use grammar::Grammar;
use state::{ StateSet, Item };

pub mod grammar;

mod state;

/// Return `true` if the input string is in the language described by
/// `grammar`, false otherwise.
pub fn recognise<S>(grammar: &Grammar, input: S) -> bool where S: AsRef<str> {
    let input = input.as_ref().chars().collect::<Vec<_>>();
    let start_symbol = grammar.start_symbol();

    // Initial state set is seeded with all of the rules that can produce the
    // start symbol
    let mut parse_state = vec![
        StateSet::new(
            Item::from_rules(grammar.get_rules_by_name(start_symbol), 0)
        )
    ];

    // len + 1 because completions still need to occur after the last character
    // is consumed, predictions can also safely occur and are useless. Any
    // attempt to scan will fail that thread of the parse.
    for current_position in 0..input.len() + 1 {

        if current_position >= parse_state.len() {
            // Ran out of state before running out of input, we didn't manage to
            // parse the whole string
            return false;
        }

        // The algorithm requires simultaneous write access to the last state
        // set in parse_state and read access to the previous state sets. Won't
        // panic because parse_state has at least one state set by construction.
        let (current_state, prev_state) = parse_state.split_last_mut().unwrap();

        let mut to_add = Vec::new();

        while let Some(&item) = current_state.next() {
            // Predictions and Completions can add new items directly to the
            // current state set. Scans (if successful) need to add items to the
            // next state set which doesn't exist yet. We batch those up and
            // create the next state set after fully processing the current
            // one. This saves additional complexity to work out whether the new
            // state set already exists (because of a previous successful
            // scan). Note: StateSet::new assumes that all of the items in its
            // to_add set are unique, this holds for items generated from scans
            // because each (already unique) item in the current state set can
            // only produce 0 or 1 item in the next, which its itself with the
            // progress marker incremented by 1 (and the symbol to the left of
            // the progress marker will always be a terminal). Predictions can
            // only generate items with progress at 0 and completions generate
            // items where the symbol to the left of the progress marker is a
            // non-terminal.
            if let Some(item) = item.parse(
                grammar,
                current_state,
                prev_state,
                &input,
                current_position
            ) {
                to_add.push(item)
            };
        }

        // Create the state set for the next iteration. If nothing is available
        // to be added the parse has failed and we'll land in the if statement
        // at the top of the loop next time around.
        if !to_add.is_empty() {
            parse_state.push(StateSet::new(to_add));
        }
    }

    // As above this won't panic because parse_state.len() >= 1 by construction.
    // The parse succeeded if there is at least one item in the last state set
    // that ...
    parse_state.last().unwrap().items().iter()
        .filter(|item| {
            // ... produces the start symbol ...
            item.rule_name() == start_symbol &&
            // ... starts at the beginning of the string (so spans the whole
            // string as we would have failed above if we failed to produce new
            // state with input left over) ...
                item.starts_at() == 0 &&
            // ... and has completed.
                item.is_complete()
        }).count() != 0
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
