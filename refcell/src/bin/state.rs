use std::cell::RefCell;

enum State { Idle, Running, Stopped }

struct Machine {
    state: RefCell<State>,
}

impl Machine {
    fn new() -> Self {
        Machine { state: RefCell::new(State::Idle) }
    }

    fn start(&self) {
        *self.state.borrow_mut() = State::Running;
        println!("Machine started!");
    }

    fn stop(&self) {
        *self.state.borrow_mut() = State::Stopped;
    }
}

fn main() {
    let machine = Machine::new();
    machine.start();
    machine.stop();
}