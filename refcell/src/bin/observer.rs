use std::cell::RefCell;
use std::rc::{Rc, Weak};

struct Subject {
    observers: RefCell<Vec<Weak<Observer>>>,
}

struct Observer {
    id: u32,
    counter: RefCell<u32>,
}

impl Subject {
    fn new() -> Rc<Self> {
        Rc::new(Subject {
            observers: RefCell::new(Vec::new()),
        })
    }

    fn add_observer(&self, obs: Rc<Observer>) {
        self.observers.borrow_mut().push(Rc::downgrade(&obs));
    }

    fn notify(&self) {
        for obs in self.observers.borrow().iter() {
            if let Some(o) = obs.upgrade() {
                o.update();
            }
        }
    }
}

impl Observer {
    fn new(id: u32, subject: &Rc<Subject>) -> Rc<Self> {
        let obs = Rc::new(Observer {
            id,
            counter: RefCell::new(0),
        });
        subject.add_observer(obs.clone());
        obs
    }

    fn update(&self) {
        *self.counter.borrow_mut() += 1;
        println!("Observer {} updated! Count = {}", self.id, *self.counter.borrow());
    }
}

fn main() {
    let subject = Subject::new();
    let _obs1 = Observer::new(1, &subject);
    let _obs2 = Observer::new(2, &subject);
    subject.notify();
}