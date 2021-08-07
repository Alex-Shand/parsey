//! Abstract Syntax Tree construction and manipulation

use std::fmt;
use std::iter::empty;
use std::rc::Rc;

use crate::grammar::{Rule, Symbol};
use crate::state::StateSet;
use crate::utils::Uncertain;

/// A parse tree node
#[derive(Clone)]
pub enum Node {
    /// An internal tree node, created from a grammar rule
    Internal {
        /// The name of the node
        name: String,
        /// Child nodes
        children: Vec<Node>
    },
    /// A leaf node, created from a terminal (Literal or OneOf)
    Leaf(char),
}

impl Node {
    /// Construct an iterator of parse trees from the Earley algorithm state
    pub(crate) fn from_parse_state<'a>(
        start_symbol: &str,
        parse_state: &[StateSet<'a>],
        input: Vec<char>,
    ) -> impl Iterator<Item = Node> + 'a {
        // To appease the borrow checker
        let len = input.len();
        NodeIterator::new(
            &Rc::new(transpose(parse_state)),
            &Rc::new(input),
            start_symbol,
            0,
            Uncertain::Known(len),
        )
    }

    // Calculate the length in characters of the node
    fn len(&self) -> usize {
        match self {
            // Leaf nodes each hold one character
            Node::Leaf(_) => 1,
            // The length of an internal node is the sum of the length of its
            // children
            Node::Internal { name: _, children } =>
                children.iter().map(Node::len).sum(),
        }
    }
}

/// Helper function to format a tree
fn format_node(f: &mut fmt::Formatter<'_>, node: &Node, id: usize) -> fmt::Result {
    let indent = if id == 0 && !f.alternate() {
        String::from(" ")
    } else {
        "    ".repeat(id)
    };
    let indent = String::from(if id == 0 { "" } else { "\n" }) + &indent;
    match node {
        Node::Leaf(c) => write!(f, "{}{}", indent, c),
        Node::Internal { name, children } => {
            write!(f, "{}{} {{", indent, name)?;
            let id = if f.alternate() { id + 1 } else { id };
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

/// Simplified version of `state::item::Item` for use in the output of
/// `transpose` (end instead of start because of transposition and no progress
/// mark because we filter out incomplete items)
#[derive(Debug, Copy, Clone)]
struct Item<'a> {
    rule: &'a Rule,
    end: usize,
}

/// Remove incomplete items from the state sets and transpose so that indexing
/// into the outer `Vec` selects on the start position of the item
fn transpose<'a>(state: &[StateSet<'a>]) -> Vec<Vec<Item<'a>>> {
    let mut result = vec![Vec::new(); state.len()];

    for (end, set) in state.iter().enumerate() {
        for item in set.items() {
            if !item.is_complete() {
                continue;
            }
            result[*item.start()].push(Item {
                rule: item.rule(),
                end,
            });
        }
    }

    result
}

/// Iterator which lazily computes possible parse trees from the transposed
/// parse state
struct NodeIterator<'a> {
    /// The current rule we're trying to produce a node for. If this ever
    /// becomes `None` the iterator ends
    current: Option<Item<'a>>,
    /// Any other rules that could produce the target node type
    candidates: Vec<Item<'a>>,
    /// A selection of candidates for the child nodes of this node, acts as a
    /// stack of child nodes in the same order the children appear in the rule
    /// definition. If we find one for each actual child we'll construct and
    /// yield a node. The iterator attached to each node is used to compute new
    /// candidates after yielding a node or running out of candidates for a
    /// later child.
    progress: Vec<(Node, Box<dyn Iterator<Item = Node> + 'a>)>,
    /// Shared reference to the transposed parse state
    parse_state: Rc<Vec<Vec<Item<'a>>>>,
    /// Shared reference to the original input (used for testing leaf node
    /// candidates)
    input: Rc<Vec<char>>,
    /// The start position of the text covered by this node in the input
    start: usize,
    /// The (potentially uncertain) end position of the text covered by this
    /// node in the input
    end: Uncertain,
}

impl<'a> NodeIterator<'a> {
    fn new(
        parse_state: &Rc<Vec<Vec<Item<'a>>>>,
        input: &Rc<Vec<char>>,
        name: &str,
        start: usize,
        end: Uncertain,
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

        // Reverse to treat it as a stack of candidates. This favors items which
        // appear earlier in the parse state. Because of how the parse state is
        // constructed this winds up favoring rules defined higher up the
        // grammar
        candidates.reverse();

        // First candidate or None if no candidate was found
        let current = candidates.pop();

        NodeIterator {
            current,
            candidates,
            progress: Vec::new(),
            parse_state: Rc::clone(parse_state),
            input: Rc::clone(input),
            start,
            end,
        }
    }

    /// Backtrack to the nearest decision point with choices remaining. May
    /// reduce the length of `self.progress`. If `self.progress` empties
    /// completely a new candidate is popped from `self.candidates` into
    /// `self.current`
    fn step(&mut self) {
        // Repeat until we find a non-exhausted decision point or choose another
        // candidate
        loop {
            if let Some((_, mut iter)) = self.progress.pop() {
                if let Some(node) = iter.next() {
                    // New decision made, pass back to the main algorithm to see
                    // if we have enough children for a node
                    self.progress.push((node, iter));
                    return;
                }
                // No more candidates for this decision point, throw it away and
                // work on the one below
            } else {
                // Exausted all of the decision points for the current
                // candidate, select the next one (or None if there isn't
                // another one) and pass back to the main algorithm to restart
                // the search
                self.current = self.candidates.pop();
                return;
            }
        }
    }
}

impl Iterator for NodeIterator<'_> {
    type Item = Node;

    #[allow(clippy::option_if_let_else)]
    fn next(&mut self) -> Option<Self::Item> {
        // Repeat until we produce a node
        loop {
            // End the iterator if there is no current candidate
            let current = self.current?;
            let body = current.rule.body();

            if self.progress.len() == body.len() {
                // Constructed a full set of children, make a copy ...
                let children = self
                    .progress
                    .iter()
                    .map(|(n, _)| n)
                    .cloned()
                    .collect::<Vec<_>>();
                // ... step to the next state ...
                self.step();
                // ... and return the node
                return Some(Node::Internal {
                    name: String::from(current.rule.name()),
                    children,
                });
            }

            // Symbol we need to produce a child node for
            let current_symbol = &body[self.progress.len()];
            // The part of the rule we don't yet have child nodes for
            let rest = &body[self.progress.len() + 1..];

            // Advance the start position by the known length of all the nodes
            // we've found so far
            let child_start = self.start + length(&self.progress[..]);
            let child_end = if self.progress.len() == body.len() - 1 {
                // If the current symbol is the last one the end marker is the
                // same as the current one
                self.end
            } else {
                // Otherwise calculate an (uncertain) lower bound on the length
                // of the remaining symbols and subtract from the end position,
                // always results in an uncertain value
                self.end - lowerbound_length(rest)
            };

            match current_symbol {
                Symbol::Rule(name) => {
                    // Dispatch to a sub iterator to handle internal nodes
                    let mut nodes = NodeIterator::new(
                        &Rc::clone(&self.parse_state),
                        &Rc::clone(&self.input),
                        name,
                        child_start,
                        child_end,
                    );

                    // Fail if we can't find at least one node
                    if let Some(node) = nodes.next() {
                        self.progress.push((node, Box::new(nodes)));
                    } else {
                        self.step();
                    }
                }
                // Terminal symbols have a have no alternate choices and fail
                // immediately if the input doesn't match what is expected
                Symbol::Literal(c) => {
                    if self.input[child_start] == *c {
                        self.progress.push((Node::Leaf(*c), Box::new(empty())));
                    } else {
                        self.step();
                    }
                }
                Symbol::OneOf(chars) => {
                    if chars.contains(&self.input[child_start]) {
                        self.progress
                            .push((Node::Leaf(self.input[child_start]), Box::new(empty())));
                    } else {
                        self.step();
                    }
                }
            }
        }
    }
}

/// Helper function to calculate the length of the nodes currently in the
/// progress stack
fn length<'a>(nodes: &[(Node, Box<dyn Iterator<Item = Node> + 'a>)]) -> usize {
    nodes.iter().map(|(n, _)| n.len()).sum()
}

/// Helper function to calculate a lower bound on the number of characters
/// needed for a sequence of symbols
fn lowerbound_length(items: &[Symbol]) -> Uncertain {
    // Assume the minimum length of all symbols is 1 TODO: This is probably
    // wrong given nullable rules
    Uncertain::Unknown(items.len())
}
