use super::Grammar;
use super::rule::Rule;
use super::symbol::Symbol;

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

struct StateSet<'a> {
    items: Vec<Item<'a>>,
    next: usize
}

impl<'a> StateSet<'a> {
    fn new(items: Vec<Item<'a>>) -> Self {
        StateSet { items, next: 0 }
    }
    
    fn next(&mut self) -> Option<&Item<'a>> {
        let current = self.next;
        self.next += 1;
        self.items.get(current)
    }

    fn add(&mut self, new_items: Vec<Item<'a>>) {
        for item in new_items {
            if !self.items.contains(&item) {
                self.items.push(item)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Item<'a> {
    rule: &'a Rule,
    state: State
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct State {
    start: usize,
    progress: usize
}

#[derive(Debug)]
enum ParseResult<'a> {
    Predict(Vec<&'a Rule>)
}

impl<'a> Item<'a> {
    fn parse<'b>(&self, grammar: &'b Grammar) -> ParseResult<'b> {
        if let Some(matcher) = self.rule.get(self.state.progress) {
            match matcher {
                Symbol::Rule(name) =>
                    ParseResult::Predict(grammar.get_rules_by_name(name)),
                _ => todo!("Scan")
            }
        } else {
            todo!("Completion")
        }
    }

    fn from_rules(rules: Vec<&'a Rule>, state: State) -> Vec<Self> {
        rules.into_iter().map(|rule| Item { rule, state }).collect::<Vec<_>>()
    }
}
