use std::cell::Cell;

struct Counter {
    n: Cell<u32>,
}

impl Counter {
    fn new() -> Self {
        Self { n: Cell::new(0) }
    }

    fn inc(&self) {
        let cur = self.n.get();
        self.n.set(cur + 1);
    }

    fn get(&self) -> u32 {
        self.n.get()
    }
}

/*

Cell<T> is a zero-cost abstraction for interior mutability. It allows you to mutate data even when you only have an immutable reference.
Important: T must implement Copy (because Cell works by copying values in/out).

Cell needed when:
- Multiple immutable references (`&self`) mutate: `Rc<Counter>` shared across threads/tasks[web:1][web:2]
- No `&mut self` possible (API requires `&self`)[web:7]
- Interior mutability without exclusive borrow

Rule of thumb

Use Cell<T> when:

    mutation is small and local,
    you don’t need to hand out references into T,
    and you’re single-threaded (or otherwise not sharing across threads).

If you need to borrow parts of T (like &mut T temporarily) you often want RefCell<T> instead; if you need cross-thread mutation, look at atomics or mutexes. [^book]
*/

fn main() {
    let c = Counter::new();
    c.inc();
    c.inc();
    assert_eq!(c.get(), 2);
}