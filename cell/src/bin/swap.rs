use std::cell::Cell;

fn main() {
    let a = Cell::new(10);
    let b = Cell::new(20);

    a.swap(&b);

    assert_eq!(a.get(), 20);
    assert_eq!(b.get(), 10);
}