use std::collections::BTreeSet;

pub fn components(edges: &[Vec<usize>]) -> Vec<BTreeSet<usize>> {
    let mut state = Tarjan {
        edges,
        index: 0,
        numbers: vec![None; edges.len()],
        low: vec![0; edges.len()],
        stack: Vec::new(),
        on_stack: vec![false; edges.len()],
        components: Vec::new(),
    };
    for node in 0..edges.len() {
        if state.numbers[node].is_none() {
            state.walk(node);
        }
    }
    state
        .components
        .sort_by_key(|component| component.iter().next().copied().unwrap_or(usize::MAX));
    state.components
}

struct Tarjan<'a> {
    edges: &'a [Vec<usize>],
    index: usize,
    numbers: Vec<Option<usize>>,
    low: Vec<usize>,
    stack: Vec<usize>,
    on_stack: Vec<bool>,
    components: Vec<BTreeSet<usize>>,
}

impl Tarjan<'_> {
    fn walk(&mut self, node: usize) {
        self.numbers[node] = Some(self.index);
        self.low[node] = self.index;
        self.index += 1;
        self.stack.push(node);
        self.on_stack[node] = true;

        for position in 0..self.edges[node].len() {
            let next = self.edges[node][position];
            match self.numbers[next] {
                None => {
                    self.walk(next);
                    self.low[node] = self.low[node].min(self.low[next]);
                }
                Some(number) if self.on_stack[next] => {
                    self.low[node] = self.low[node].min(number);
                }
                Some(_) => {}
            }
        }

        if self.numbers[node] == Some(self.low[node]) {
            let mut component = BTreeSet::new();
            while let Some(member) = self.stack.pop() {
                self.on_stack[member] = false;
                component.insert(member);
                if member == node {
                    break;
                }
            }
            self.components.push(component);
        }
    }
}
