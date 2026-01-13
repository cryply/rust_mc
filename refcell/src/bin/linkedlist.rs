use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct Node {
    value: i32,
    next: RefCell<Option<Rc<Node>>>,
}

impl Node {
    fn new(val: i32) -> Rc<Self> {
        Rc::new(Node {
            value: val,
            next: RefCell::new(None),
        })
    }

    fn push(&self, node: Rc<Node>) {
        if self.next.borrow().is_none() {
            *self.next.borrow_mut() = Some(node);
            return;
        }
        // recurse on next node
        if let Some(next) = self.next.borrow().as_ref() {
            next.push(node);
        }
    }
}

fn main() {
    let head = Node::new(1);
    head.push(Node::new(2));
    head.push(Node::new(3));
    println!("{:?}", head);
}
