fn leaf(name: &str) -> Node {
    Node {
        name: String::from(name),
        children: Vec::new()
    }
}

fn yield_leaf<'a, A>(name: &'a str) -> Generator<'_, A, Node> where A: Send + 'a {
    Gn::new_scoped(move |mut s| {
        let _ = s.yield_(leaf(name));
        done!()
    })
}

fn trees_from_symbol(symbol: &Symbol, input: &[char]) -> impl Iterator<Item=Node> {
    match symbol {
        Symbol::Rule(name) => todo!(),
        Symbol::Literal(c) => todo!(),
        Symbol::OneOf(chars) => todo!()
    }
    vec![].into_iter()
}

fn trees_from_item<'a>(item: Item<'a>, input: &[char]) -> impl Iterator<Item=Node> + 'a {
    println!("{:?}", item);
    let name = item.rule.name();
    let body = item.rule.body();

    if body.is_empty() {
        return yield_leaf(name);
    }

    let mut first_child = trees_from_symbol(&body[0], input);
    if let Some(candidate) = first_child.next() {
        let mut stack = VecDeque::with_capacity(body.len());
        stack.push_front((candidate, first_child));

        Gn::new_scoped(move |mut s| {
            while !stack.is_empty() {
                fill_stack(&mut stack, &body);
                assert!(stack.len() == body.len(), "Decision stack has {} elements, expected {}", stack.len(), body.len());
                let _ = s.yield_(Node {
                    name: String::from(name),
                    children: stack.iter().map(|(c, _)| c.clone()).rev().collect::<Vec<_>>()
                });
                make_decision(&mut stack);
            }
            done!()
        })
    } else {
        yield_leaf(name)
    }
}

fn fill_stack(stack: &mut VecDeque<(Node, impl Iterator<Item=Node>)>, symbols: &[Symbol]) {
}

fn make_decision(stack: &mut VecDeque<(Node, impl Iterator<Item=Node>)>) {
    assert!(!stack.is_empty(), "Decision stack is empty, should have bailed by now");
    while !stack.is_empty() {
        let (_, mut candidates) = stack.pop_front().unwrap();
        if let Some(candidate) = candidates.next() {
            stack.push_front((candidate, candidates));
            return;
        }
    }
}

fn build_parse_trees<'a>(
    state: Vec<Vec<Item<'a>>>,
    input: &'a [char],
    name: &'a str,
    start: usize,
    end: Option<usize>
) -> impl Iterator<Item=Node> + 'a {
    Gn::new_scoped(move |mut s| {
        let candidates = state[start].iter().copied().filter(|item| item.rule.name() == name);
        let candidates = if let Some(end) = end {
            candidates.filter(|item| item.end == end).collect::<Vec<_>>()
        } else {
            candidates.collect::<Vec<_>>()
        };

        for candidate in candidates {
            for node in trees_from_item(candidate, input) {
                let _ = s.yield_(node);
            }
        }

        done!()
    })
}
