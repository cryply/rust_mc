use std::cell::RefCell;
use std::ops::Deref;

struct MyBox<T>(RefCell<T>);

impl<T> MyBox<T> {
    fn new(value: T) -> Self {
        MyBox(RefCell::new(value))
    }
}

impl<T> Deref for MyBox<T> {
    type Target = RefCell<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn main() {
    let num = MyBox::new(5);
    *num.borrow_mut() += 10;      // Mutate via MyBox
    println!("{}", *num.borrow()); // 15
}