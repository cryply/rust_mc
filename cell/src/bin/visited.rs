use std::cell::Cell;

struct Node {
    visited: Cell<bool>,
}

impl Node {
    fn new() -> Self {
        Self { visited: Cell::new(false) }
    }

    fn mark_visited(&self) {
        self.visited.set(true);
    }

    fn is_visited(&self) -> bool {
        self.visited.get()
    }
}

fn main() {
    let n = Node::new();
    assert!(!n.is_visited());
    n.mark_visited();
    assert!(n.is_visited());
}