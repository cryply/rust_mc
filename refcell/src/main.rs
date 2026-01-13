/*
RefCell<T> provides interior mutability — you can mutate data even when you only have an immutable reference (&self). It enforces Rust’s borrowing rules at runtime (panics if rules are broken).

⚠️ Important: RefCell is single‑threaded only. For multi‑threaded cases, use Mutex/RwLock.
*/


use std::cell::RefCell;

struct Counter {
    value: RefCell<i32>,
}

impl Counter {
    fn new() -> Self {
        Counter { value: RefCell::new(0) }
    }

    // `increment` takes `&self` (immutable), but mutates `value`!
    fn increment(&self) {
        *self.value.borrow_mut() += 1;   // Get mutable ref
    }

    fn get(&self) -> i32 {
        *self.value.borrow()             // Get immutable ref
    }
}

fn main() {
    let counter = Counter::new();
    counter.increment();
    counter.increment();
    println!("Count: {}", counter.get()); // Count: 2
}