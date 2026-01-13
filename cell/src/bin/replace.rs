use std::cell::Cell;

struct IdGen {
    next: Cell<u32>,
}

impl IdGen {
    fn new(start: u32) -> Self {
        Self { next: Cell::new(start) }
    }

    fn alloc(&self) -> u32 {
        // replace() sets the new value and returns the old one
        let cur = self.next.get();
        self.next.replace(cur + 1)
    }
}

fn main() {
    let gen1 = IdGen::new(100);
    assert_eq!(gen1.alloc(), 100);
    assert_eq!(gen1.alloc(), 101);
}