use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

struct Philosopher {
    id: u32,
    name: String,
}

impl Philosopher {
    // Implement dining logic
    fn new(id: u32, name: String) -> Self {
        Self{id, name}
    }
    
    fn eat(&self,   c: Arc<Mutex<u32>>) {
        let mut guard = c.lock().unwrap();
        *guard += 1;
        println!("Philosopher id: {} chopstick id:{} EATING", self.id, guard);

    }
    
}

fn main() {
    // Threads pass messages 
    // let (tx, rx) = mpsc::channel();
    
    // Mutex protects share state
    let counter = Arc::new(Mutex::new(0u32)); 
    
    // Rayon parallelizes 
    // (0..10).into_par_iter().for_each(|x| {
    //     // Do work in parallel    
    // });
    
    // Spawn philosopher threads
    let chopsticks = Arc::new(Mutex::new(0)); 
    let philosophers = vec![
        Philosopher::new(1, "Pluto".to_string()),
        Philosopher::new(2, "Socratus".to_string()),
        Philosopher::new(3, "Descart".to_string()),
        Philosopher::new(4, "Marc Avrelius".to_string()),
        Philosopher::new(5, "Heraclites".to_string()),
   
    ];

    let mut handles = vec![];
    for p in philosophers {
        let c = chopsticks.clone();
        handles.push(thread::spawn(move || {
            p.eat(c); 
        }));
    }

    for h in handles {
        h.join();
    }
}