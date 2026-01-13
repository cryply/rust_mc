use std::cell::Cell;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum State {
    Init,
    Running,
    Done,
}

struct Job {
    state: Cell<State>,
}

impl Job {
    fn new() -> Self {
        Self { state: Cell::new(State::Init) }
    }

    fn start(&self) {
        if self.state.get() == State::Init {
            self.state.set(State::Running);
        }
    }

    fn finish(&self) {
        if self.state.get() == State::Running {
            self.state.set(State::Done);
        }
    }

    fn state(&self) -> State {
        self.state.get()
    }
}

fn main() {
    let j = Job::new();
    assert_eq!(j.state(), State::Init);
    j.start();
    assert_eq!(j.state(), State::Running);
    j.finish();
    assert_eq!(j.state(), State::Done);
}