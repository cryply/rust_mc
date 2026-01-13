use std::cell::Cell;

struct Expensive {
    input: u64,
    cached: Cell<Option<u64>>, // Option<u64> is Copy
}

impl Expensive {
    fn new(input: u64) -> Self {
        Self { input, cached: Cell::new(None) }
    }

    fn compute(&self) -> u64 {
        if let Some(v) = self.cached.get() {
            return v;
        }

        // pretend this is expensive
        let v = self.input * self.input + 1;

        self.cached.set(Some(v));
        v
    }
}

fn main() {
    let e = Expensive::new(12);
    assert_eq!(e.compute(), 145);
    assert_eq!(e.compute(), 145); // second call hits cache
}