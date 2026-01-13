use std::cell::RefCell;
use std::rc::Rc;

struct Node {
    name: String,
    children: RefCell<Vec<Rc<Node>>>,
}

impl Node {
    fn new(name: &str) -> Rc<Self> {
        Rc::new(Node {
            name: name.to_string(),
            children: RefCell::new(Vec::new()),
        })
    }

    fn add_child(&self, child: Rc<Node>) {
        self.children.borrow_mut().push(child); // Mutate children list
    }
}

fn main() {
    let root = Node::new("root");
    let child = Node::new("child");
    root.add_child(child.clone());
}