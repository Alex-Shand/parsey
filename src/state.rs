use super::grammar::{ Grammar, Rule, Symbol };

pub struct StateSet<'a> {
    items: Vec<Item<'a>>,
    next: usize
}

impl<'a> StateSet<'a> {
    pub fn new(items: Vec<Item<'a>>) -> Self {
        StateSet { items, next: 0 }
    }
    
    pub fn next(&mut self) -> Option<&Item<'a>> {
        let current = self.next;
        self.next += 1;
        self.items.get(current)
    }

    pub fn add(&mut self, new_items: Vec<Item<'a>>) {
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
    pub start: usize,
    pub progress: usize
}

#[derive(Debug)]
pub enum ParseResult<'a> {
    Predict(Vec<&'a Rule>),
    Scan(Option<Item<'a>>)
}

impl<'a> Item<'a> {
    pub fn parse<'b>(
        &self,
        grammar: &'b Grammar,
        next_char: char
    ) -> ParseResult<'b> {
        if let Some(matcher) = self.rule.get(self.state.progress) {
            match matcher {
                Symbol::Rule(name) =>
                    ParseResult::Predict(grammar.get_rules_by_name(name)),
                Symbol::Literal(c) =>
                    ParseResult::Scan(
                        if *c == next_char {
                            todo!("Scan Literal")
                        } else {
                            None
                        }
                    ),
                Symbol::OneOf(_) => todo!("Scan OneOf")
            }
        } else {
            todo!("Completion")
        }
    }

    pub fn from_rules(rules: Vec<&'a Rule>, state: State) -> Vec<Self> {
        rules.into_iter().map(|rule| Item { rule, state }).collect::<Vec<_>>()
    }
}
