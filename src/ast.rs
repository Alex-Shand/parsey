use std::collections::VecDeque;
use std::rc::Rc;
use std::iter::{once, empty};
use std::fmt;

use search::BacktrackingSearch;

use crate::state::StateSet;
use crate::grammar::{Rule, Symbol};

#[derive(Clone)]
pub enum Node {
    Internal {
        name: String,
        children: Vec<Node>
    },
    Leaf(char)
}

#[derive(Debug, Copy, Clone)]
enum End {
    Exactly(usize),
    Before(usize)
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
            End::Exactly(len)
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

#[derive(Debug)]
struct NodeIterator<'a> {
    candidates: VecDeque<Item<'a>>,
    search: BacktrackingSearch<'a, Symbol, Node>,
    parse_state: Rc<Vec<Vec<Item<'a>>>>,
    input: Rc<Vec<char>>,
    start: usize,
    end: End
}

impl<'a> NodeIterator<'a> {
    fn new(
        parse_state: Rc<Vec<Vec<Item<'a>>>>,
        input: Rc<Vec<char>>,
        name: &str,
        start: usize,
        end: End
    ) -> Self {
        let candidates = parse_state[start].iter().copied().filter(|item| item.rule.name() == name);
        let candidates = match end {
            End::Exactly(e) => candidates.filter(|item| item.end == e).collect::<VecDeque<_>>(),
            End::Before(e) => candidates.filter(|item| item.end < e).collect::<VecDeque<_>>()
        };
        let mut this = NodeIterator {
            candidates,
            search: BacktrackingSearch::default(),
            parse_state,
            input,
            start,
            end
        };
        if !this.candidates.is_empty() {
            this.init_search();
        }
        this
     }

    fn init_search(&mut self) {
        if self.candidates.is_empty() {
            self.search = BacktrackingSearch::default();
            return;
        }
        let candidate = self.candidates[0];
        let body = candidate.rule.body();
        let parse_state = Rc::clone(&self.parse_state);
        let input = Rc::clone(&self.input);
        let start = self.start;
        let end = self.end;
        let body_len = body.len();
        self.search = BacktrackingSearch::new(
            body,
            move |symbol, nodes: &[Node]| {
                let offset: usize = nodes.iter().map(|n| n.len()).sum();
                let start = start + offset;

                let end = match end {
                    End::Exactly(e) => if nodes.len() < body.len() - 1 {
                        End::Before(e)
                    } else {
                        End::Exactly(e)
                    }
                    e @ End::Before(_) => e
                };

                match symbol {
                    Symbol::Rule(name) => Box::new(
                        NodeIterator::new(
                            Rc::clone(&parse_state),
                            Rc::clone(&input),
                            name,
                            start,
                            end
                        )
                    ),
                    Symbol::Literal(c) => {
                        println!("Literal Start: {}, Char: {}, Expected: {}", start, input[start], c);
                        if input[start] == '+' {
                            println!("{:?}", parse_state);
                        }
                        if *c == input[start] {
                            Box::new(once(Node::Leaf(*c)))
                        } else {
                            Box::new(empty())
                        }
                    },
                    Symbol::OneOf(chars) => {
                        println!("OneOf Start: {}, Char: {}, Expected: {:?}", start, input[start], chars);
                        let c = input[start];
                        if chars.contains(&c) {
                            Box::new(once(Node::Leaf(c)))
                        } else {
                            Box::new(empty())
                        }
                    }
                }
            }
        )
    }
}

impl Iterator for NodeIterator<'_> {
    type Item = Node;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.candidates.is_empty() {
                return None;
            }
            let candidate = self.candidates[0];
            if let Some(children) = self.search.next() {
                return Some(
                    Node::Internal {
                        name: String::from(candidate.rule.name()),
                        children
                    }
                );
            }
            let _ = self.candidates.pop_front();
            self.init_search();
        }
    }
}
