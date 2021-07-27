use std::collections::VecDeque;
use std::rc::Rc;
use std::iter::{once, empty};
use std::fmt;

use search::BacktrackingSearch;

use crate::state::StateSet;
use crate::grammar::{Rule, Symbol};

#[derive(Debug, Copy, Clone)]
enum Uncertain {
    Known(usize),
    Unknown(usize)
}

impl Uncertain {
    fn subtract(self, other: Self) -> Self {
        match self {
            Uncertain::Known(a) => match other {
                Uncertain::Known(b) => Uncertain::Known(a - b),
                Uncertain::Unknown(b) => Uncertain::Unknown(a - b)
            },
            Uncertain::Unknown(a) => match other {
                Uncertain::Known(b) => Uncertain::Unknown(a - b),
                Uncertain::Unknown(b) => Uncertain::Unknown(a - b)
            }
        }
    }
}

#[derive(Clone)]
pub enum Node {
    Internal {
        name: String,
        children: Vec<Node>
    },
    Leaf(char)
}

impl Node {
    pub(crate) fn from_parse_state<'a>(
        start_symbol: &str,
        parse_state: Vec<StateSet<'a>>,
        input: Vec<char>
    ) -> impl Iterator<Item=Node> + 'a {
        let len = input.len();
        NodeIterator::new(
            Rc::new(transpose(parse_state)),
            Rc::new(input),
            start_symbol,
            0,
            Uncertain::Known(len)
        )
    }

    fn len(&self) -> usize {
        match self {
            Node::Leaf(_) => 1,
            Node::Internal{name, children} => children.iter().map(|n| n.len()).sum()
        }
    }
}

fn format_node(f: &mut fmt::Formatter<'_>, node: &Node, id: usize) -> fmt::Result {
    let indent = if id == 0 && !f.alternate() {
        String::from(" ")
    } else {
        "    ".repeat(id)
    };
    let indent = String::from("\n") + &indent;
    match node {
        Node::Leaf(c) => write!(f, "{}{}", indent, c),
        Node::Internal{name, children} => {
            write!(f, "{}{} {{", indent, name)?;
            let id = if f.alternate() {
                id + 1
            } else {
                id
            };
            for child in children {
                format_node(f, child, id)?;
            }
            write!(f, "{}}}", indent)
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_node(f, self, 0)
    }
}

#[derive(Debug, Copy, Clone)]
struct Item<'a> {
    rule: &'a Rule,
    end: usize
}

fn transpose(state: Vec<StateSet<'_>>) -> Vec<Vec<Item<'_>>> {
    let mut result = vec![Vec::new(); state.len()];

    for (end, set) in state.iter().enumerate() {
        for item in set.items() {
            if !item.is_complete() {
                continue;
            }
            result[*item.start()].push(Item { rule: item.rule(), end })
        }
    }

    result
}

struct NodeIterator<'a> {
    current: Option<Item<'a>>,
    candidates: Vec<Item<'a>>,
    progress: Vec<(Node, Box<dyn Iterator<Item=Node> + 'a>)>,
    parse_state: Rc<Vec<Vec<Item<'a>>>>,
    input: Rc<Vec<char>>,
    start: usize,
    end: Uncertain
}

impl<'a> NodeIterator<'a> {
    fn new(
        parse_state: Rc<Vec<Vec<Item<'a>>>>,
        input: Rc<Vec<char>>,
        name: &str,
        start: usize,
        end: Uncertain
    ) -> Self {
        let mut candidates =
            // Check for candidates that start at the correct position ...
            parse_state[start].iter()
            // ... with a matching name ...
            .filter(|item| item.rule.name() == name)
            // ... and isn't too long
            .filter(|item| match end {
                Uncertain::Known(e) => item.end == e,
                Uncertain::Unknown(e) => item.end <= e
            })
            .copied().collect::<Vec<_>>();

        // Reverse to treat it as a stack of candidates
        candidates.reverse();

        // First candidate or None if no candidate was found
        let current = candidates.pop();

        NodeIterator {
            current,
            candidates,
            progress: Vec::new(),
            parse_state: Rc::clone(&parse_state),
            input: Rc::clone(&input),
            start,
            end
        }
    }

    fn step(&mut self) {
        loop {
            if let Some((_, mut iter)) = self.progress.pop() {
                if let Some(node) = iter.next() {
                    self.progress.push((node, iter));
                    return
                }
            } else {
                self.current = self.candidates.pop();
                return
            }
        }
    }
}

impl Iterator for NodeIterator<'_> {
    type Item = Node;
    fn next(&mut self) -> Option<Self::Item> {
        // Repeat until we produce a node
        loop {
            // End the iterator if there is no current candidate
            let current = self.current?;
            let body = current.rule.body();

            if self.progress.len() == body.len() {
                // Constructed a full set of children, make a copy ...
                let children = self.progress.iter().map(|(n, _)| n).cloned().collect::<Vec<_>>();
                // ... step to the next state ...
                self.step();
                // ... and return the node
                return Some(Node::Internal {
                    name: String::from(current.rule.name()),
                    children
                })
            }

            // Symbol we need to produce a node for
            let current_symbol = &body[self.progress.len()];
            // The part of the rule we don't yet have nodes for
            let rest = &body[self.progress.len()+1..];

            // Advance the start position by the known length of all the nodes
            // we've found so far
            let child_start = self.start + length(&self.progress[..]);
            let child_end = if self.progress.len() == body.len() - 1 {
                // The current symbol is the last one the end marker is the same
                // as the current one
                self.end
            } else {
                // Otherwise calculate an (uncertain) lower bound on the length
                // of the remaining symbols and subtract from the end position,
                // always results in an uncertain value
                self.end.subtract(lowerbound_length(rest))
            };

            match current_symbol {
                Symbol::Rule(name) => {
                    // Dispatch to a sub iterator to handle internal nodes
                    let mut nodes = NodeIterator::new(
                        Rc::clone(&self.parse_state),
                        Rc::clone(&self.input),
                        &name,
                        child_start,
                        child_end
                    );
                    if let Some(node) = nodes.next() {
                        self.progress.push((node, Box::new(nodes)))
                    } else {
                        self.step()
                    }
                }
                Symbol::Literal(c) => {
                    if self.input[child_start] == *c {
                        self.progress.push((Node::Leaf(*c), Box::new(empty())));
                    } else {
                        self.step()
                    }
                }
                Symbol::OneOf(chars) => {
                    if chars.contains(&self.input[child_start]) {
                        self.progress.push((Node::Leaf(self.input[child_start]), Box::new(empty())));
                    } else {
                        self.step()
                    }
                }
            }
        }
    }
}

fn length<'a>(nodes: &[(Node, Box<dyn Iterator<Item=Node> + 'a>)]) -> usize {
    nodes.iter().map(|(n, _)| n.len()).sum()
}

fn lowerbound_length(items: &[Symbol]) -> Uncertain {
    Uncertain::Unknown(items.len())
}
