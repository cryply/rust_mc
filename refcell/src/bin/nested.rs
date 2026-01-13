use std::cell::RefCell;

struct Inner {
    value: RefCell<i32>,
}

struct Outer {
    inner: Inner,
}

impl Outer {
    fn new() -> Self {
        Outer {
            inner: Inner { value: RefCell::new(10) },
        }
    }

    fn increment(&self) {
        *self.inner.value.borrow_mut() += 1;
    }
}

fn main() {
    let obj = Outer::new();
    obj.increment();
    println!("{}", *obj.inner.value.borrow()); // 11
}