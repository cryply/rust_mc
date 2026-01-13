use std::cell::Cell;
use std::rc::Rc;

struct Tracker {
    hits: Cell<u32>,
}

impl Tracker {
    fn new() -> Self {
        Self { hits: Cell::new(0) }
    }

    fn hit(&self) {
        self.hits.set(self.hits.get() + 1);
    }

    fn hits(&self) -> u32 {
        self.hits.get()
    }
}

fn main() {
    let t = Rc::new(Tracker::new());
    let t2 = Rc::clone(&t);

    t.hit();
    t2.hit();
    t2.hit();

    assert_eq!(t.hits(), 3);
}