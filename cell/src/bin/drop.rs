use std::cell::Cell;

struct Guard {
    armed: Cell<bool>,
}

impl Guard {
    fn new() -> Self {
        Self { armed: Cell::new(true) }
    }

    // Can be called even if you only have &Guard (shared reference)
    fn disarm(&self) {
        self.armed.set(false);
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        if self.armed.get() {
            // cleanup action
            // (printing just for demo)
            println!("Guard dropped while still armed!");
        }
    }
}

fn main() {
    let g = Guard::new();
    g.disarm(); // prevent cleanup
}