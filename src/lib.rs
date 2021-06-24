//! Earley parser
#![warn(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(noop_method_call)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_results)]
#![deny(missing_debug_implementations)]
#![deny(missing_copy_implementations)]
//#![deny(dead_code)]
#![warn(clippy::pedantic)]

use grammar::Grammar;
use state::{Item, StateSet};

pub mod grammar;

mod state;
mod utils;
pub use utils::NonEmptyHashSet;

fn expand_input<S>(input: S) -> Vec<char>
where
    S: AsRef<str>,
{
    input.as_ref().chars().collect()
}

fn build_parse_state<'a>(
    start_symbol: &'a str,
    grammar: &'a Grammar,
    input: &'a [char],
) -> Result<Vec<StateSet<'a>>, &'a [char]> {
    // Initial state set is seeded with all of the rules that can produce the
    // start symbol
    let mut parse_state = vec![StateSet::new(Item::from_rules(
        grammar.get_rules_by_name(start_symbol),
        0,
    ))];

    // len + 1 because completions still need to occur after the last character
    // is consumed, predictions can also safely occur and are useless. Any
    // attempt to scan will fail that thread of the parse.
    for current_position in 0..=input.len() {
        if current_position >= parse_state.len() {
            // Ran out of state before running out of input, we didn't manage to
            // parse the whole string (use current_position - 1 because the
            // error actually occurred in the previous iteration of the loop,
            // safe because parse_state.len() is always >= 1)
            return Err(&input[current_position - 1..input.len()]);
        }

        // The algorithm requires simultaneous write access to the last state
        // set in parse_state and read access to the previous state sets. Won't
        // panic because parse_state has at least one state set by construction.
        let (current_state, prev_state) = parse_state.split_last_mut().unwrap();

        let mut to_add = Vec::new();

        while let Some(item) = current_state.next() {
            // Predictions and completions can add new items directly to the
            // current state set. Scans (if successful) need to add items to the
            // next state set which doesn't exist yet. We batch those up and
            // create the next state set after fully processing the current
            // one. This saves additional complexity to work out whether the new
            // state set already exists (because of a previous successful
            // scan). Note: StateSet::new assumes that all of the items in its
            // to_add set are unique, this holds for items generated from scans
            // because each (already unique) item in the current state set can
            // only produce 0 or 1 item in the next, which is itself with the
            // progress marker incremented by 1 (and the symbol to the left of
            // the progress marker will always be a terminal). Predictions can
            // only generate items with progress at 0 and completions generate
            // items where the symbol to the left of the progress marker is a
            // non-terminal.
            if let Some(item) =
                item.parse(grammar, current_state, prev_state, &input, current_position)
            {
                to_add.push(item)
            };
        }

        // Create the state set for the next iteration. If nothing is available
        // we're either on the last state set (current_position == input.len())
        // and the loop is about to terminate or the parse has failed and we'll
        // land in the if statement at the top of the loop next time around.
        if !to_add.is_empty() {
            parse_state.push(StateSet::new(to_add));
        }
    }

    Ok(parse_state)
}

/// Return `true` if the input string is in the language described by
/// `grammar`, false otherwise.
#[allow(clippy::missing_panics_doc)]
pub fn recognise<S>(grammar: &Grammar, input: S) -> bool
where
    S: AsRef<str>,
{
    let input = expand_input(input);
    let start_symbol = grammar.start_symbol();

    // Build parse state will succeed if it can produce a state set for every
    // character in the input. This doesn't necessarily mean the parse succeeded
    if let Ok(parse_state) = build_parse_state(start_symbol, grammar, &input) {
        // The parse succeeded if there is at least one item in the last state set
        // that ...
        parse_state
            .last()
            .unwrap()
            .items()
            .iter()
            .filter(|item| {
                // ... produces the start symbol ...
                item.rule_name() == start_symbol &&
                // ... starts at the beginning of the string ...
                    item.start() == &0 &&
                // ... and has completed.
                    item.is_complete()
            })
            .count()
            != 0
    } else {
        false
    }
}

syntax_abuse::tests! {

    testdata! {
        ARITH : Grammar = grammar! {
            Sum -> Sum ["+-"] Product;
            Sum -> Product;
            Product -> Product ["*/"] Factor;
            Product -> Factor;
            Factor -> "(" Sum ")";
            Factor -> Number;
            Number -> ["0123456789"] Number;
            Number -> ["0123456789"];
        };
        EMPTY: Grammar = grammar! {
            Empty ->;
        };
        ALMOST_EMPTY: Grammar = grammar! {
            Rule -> "Rule" Empty;
            Empty ->;
        };
        LOOP : Grammar = grammar! {
            A ->;
            A -> B;
            B -> A
        };
    }

    tests! {
        build_parse_state:

        macro_rules! testcase {
            ($name : ident, $grammar : expr, $input : expr, $expected : expr) => {
                #[test]
                fn $name() {
                    let input = expand_input($input);
                    assert_eq!(
                        build_parse_state($grammar.start_symbol(), &$grammar, &input),
                        $expected
                    )
                }
            }
        }

        macro_rules! err {
            ($string: expr) => {
                Err(&$string.chars().collect::<Vec<_>>()[..])
            }
        }

        macro_rules! flatvec {
            () => { vec![] };
            ($($iters: expr),* $(,)?) => {
                vec![$($iters),*].into_iter().flatten().collect::<Vec<_>>()
            }
        }

        fn make_items<'a ,'b>(
            grammar: &'a Grammar,
            name: &'b str,
            start: usize,
            progress: usize
        ) -> Vec<Item<'a>> {
            grammar.get_rules_by_name(name).into_iter()
                .map(move |rule| Item::from_parts(rule, start, progress))
                .collect()
        }

        fn make_item(
            grammar: &Grammar,
            idx: usize,
            start: usize,
            progress: usize
        ) -> Vec<Item<'_>> {
            vec![Item::from_parts(grammar.index(idx), start, progress)]
        }

        testcase! {
            success,
            ARITH,
            "1+2",
            Ok(vec![
                StateSet::exhausted(flatvec![
                    make_items(&ARITH, "Sum", 0, 0),
                    make_items(&ARITH, "Product", 0, 0),
                    make_items(&ARITH, "Factor", 0, 0),
                    make_items(&ARITH, "Number", 0, 0)
                ]),
                StateSet::exhausted(flatvec![
                    make_items(&ARITH, "Number", 0, 1),
                    make_items(&ARITH, "Number", 1, 0),
                    make_item(&ARITH, 5, 0, 1),
                    make_item(&ARITH, 3, 0, 1),
                    make_item(&ARITH, 1, 0, 1),
                    make_item(&ARITH, 2, 0, 1),
                    make_item(&ARITH, 0, 0, 1)
                ]),
                StateSet::exhausted(flatvec![
                    make_item(&ARITH, 0, 0, 2),
                    make_items(&ARITH, "Product", 2, 0),
                    make_items(&ARITH, "Factor", 2, 0),
                    make_items(&ARITH, "Number", 2, 0)
                ]),
                StateSet::exhausted(flatvec![
                    make_items(&ARITH, "Number", 2, 1),
                    make_items(&ARITH, "Number", 3, 0),
                    make_item(&ARITH, 5, 2, 1),
                    make_item(&ARITH, 3, 2, 1),
                    make_item(&ARITH, 0, 0, 3),
                    make_item(&ARITH, 2, 2, 1),
                    make_item(&ARITH, 0, 0, 1),
                ])
            ])
        }

        testcase! {
            truncated_input,
            ARITH,
            "1+",
            Ok(vec![
                StateSet::exhausted(flatvec![
                    make_items(&ARITH, "Sum", 0, 0),
                    make_items(&ARITH, "Product", 0, 0),
                    make_items(&ARITH, "Factor", 0, 0),
                    make_items(&ARITH, "Number", 0, 0)
                ]),
                StateSet::exhausted(flatvec![
                    make_items(&ARITH, "Number", 0, 1),
                    make_items(&ARITH, "Number", 1, 0),
                    make_item(&ARITH, 5, 0, 1),
                    make_item(&ARITH, 3, 0, 1),
                    make_item(&ARITH, 1, 0, 1),
                    make_item(&ARITH, 2, 0, 1),
                    make_item(&ARITH, 0, 0, 1)
                ]),
                StateSet::exhausted(flatvec![
                    make_item(&ARITH, 0, 0, 2),
                    make_items(&ARITH, "Product", 2, 0),
                    make_items(&ARITH, "Factor", 2, 0),
                    make_items(&ARITH, "Number", 2, 0)
                ])
            ])
        }

        testcase! {
            invalid_character,
            ARITH,
            "1%2",
            err!("%2")
        }

        testcase! {
            valid_character_in_the_wrong_place,
            ARITH,
            "+1",
            err!("+1")
        }

        testcase! {
            loop_grammar,
            LOOP,
            "",
            Ok(vec![
                StateSet::exhausted(flatvec![
                    make_items(&LOOP, "A", 0, 0),
                    make_items(&LOOP, "B", 0, 0),
                    make_items(&LOOP, "B", 0, 1),
                    make_item(&LOOP, 1, 0, 1)
                ])
            ])
        }
    }

    tests! {
        recogniser:

        testcase! {
            success,
            recognise(&ARITH, "1+2+3-4+5*(6+7)/106"),
            true
        }

        testcase! {
            truncated_input,
            recognise(&ARITH, "1+"),
            false
        }

        testcase! {
            invalid_character,
            recognise(&ARITH, "1%2"),
            false
        }

        testcase! {
            valid_character_in_the_wrong_place,
            recognise(&ARITH, "+1"),
            false
        }

        testcase! {
            empty_success,
            recognise(&EMPTY, ""),
            true
        }

        testcase! {
            empty_fail,
            recognise(&EMPTY, " "),
            false
        }

        testcase! {
            almost_empty,
            recognise(&ALMOST_EMPTY, "Rule"),
            true
        }
    }
}
