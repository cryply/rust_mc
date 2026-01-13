use std::cell::Cell;

fn main() {
    let name: Cell<String> = Cell::new("Ada".to_string());

    // take() moves the value out, leaving Default::default() behind.
    let old = name.take();
    assert_eq!(old, "Ada");
    assert_eq!(name.into_inner(), String::new());
}