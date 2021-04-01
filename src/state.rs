use std::fmt;

use super::grammar::{ Grammar, Rule, Symbol };

#[derive(Debug)]
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

    pub fn items(&self) -> &Vec<Item<'a>> {
        &self.items
    }
}

impl fmt::Display for StateSet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.items.iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Item<'a> {
    rule: &'a Rule,
    state: State
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct State {
    pub start: usize,
    pub progress: usize
}

impl<'a> Item<'a> {
    pub fn parse(
        &self,
        grammar: &'a Grammar,
        current_state: &mut StateSet<'a>,
        prev_state: &[StateSet<'a>],
        input: &[char],
        current_position: usize
    ) -> Option<Item<'a>> {
        if let Some(matcher) = self.rule.get(self.state.progress) {
            match matcher {
                Symbol::Rule(name) => {
                    current_state.add(
                        Item::from_rules(
                            grammar.get_rules_by_name(name),
                            current_position
                        )
                    );
                    None
                }
                Symbol::Literal(c) =>
                    self.scan(input.get(current_position), |next| next == c),
                Symbol::OneOf(cs) => self.scan(
                    input.get(current_position), |next| cs.contains(next)
                )
            }
        } else {
            let completed = self.rule.name();
            current_state.add(
                prev_state[self.state.start].items().iter()
                    .filter_map(|item| {
                        item.next_name()
                            .filter(|name| *name == completed)
                            .map(|_| item.advanced())
                    }).collect::<Vec<Item>>()
            );
            None
        }
    }

    fn scan(
        &self,
        expected: Option<&char>,
        pred: impl FnOnce(&char) -> bool
    ) -> Option<Self> {
        expected.copied().filter(pred).map(|_| self.advanced())
    }

    pub fn rule_name(&self) -> &str {
        &self.rule.name()
    }

    pub fn is_complete(&self) -> bool {
        self.state.progress >= self.rule.body().len()
    }

    pub fn starts_at(&self) -> usize {
        self.state.start
    }
    
    pub fn next_name(&self) -> Option<&str> {
        if let Some(symbol) = self.rule.get(self.state.progress) {
            match symbol {
                Symbol::Rule(name) => Some(name),
                _ => None
            }
        } else {
            None
        }
    }

    pub fn from_rules(rules: Vec<&'a Rule>, start: usize) -> Vec<Self> {
        rules.into_iter().map(|rule| Item { rule, state: State { start, progress: 0 }}).collect::<Vec<_>>()
    }

    pub fn advanced(&self) -> Self {
        let mut new = *self;
        new.state.progress += 1;
        new
    }
}

impl fmt::Display for Item<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut body = self.rule.body().to_owned();
        body.insert(self.state.progress, Symbol::Rule(String::from("\u{25CF}")));
        let body = body.into_iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" ");
        write!(f, "{} -> {} ({})", self.rule.name(), body, self.state.start)
    }
}
