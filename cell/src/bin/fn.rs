use std::cell::Cell;

fn call_three_times(f: impl Fn()) {
    f();
    f();
    f();
}

fn main() {
    let hits = Cell::new(0u32);

    // This closure can still be `Fn` because it doesn't mutate captured variables
    // in Rust's normal sense; it mutates *through Cell*.
    call_three_times(|| hits.set(hits.get() + 1));

    assert_eq!(hits.get(), 3);
}