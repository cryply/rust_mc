use std::sync::atomic::{AtomicI8, Ordering};
use std::sync::Arc;
use std::thread;

const IDLE: i8 = 0;
const RUNNING: i8 = 1;
const FINISHED: i8 = 2;

fn main() {
    let state = Arc::new(AtomicI8::new(IDLE));
    let mut handles = vec![];

    // A worker that tries to start the job
    for i in 0..3 {
        let state_clone = Arc::clone(&state);
        let handle = thread::spawn(move || {
            println!("Worker {}: trying to start job.", i);
            // Try to transition from IDLE to RUNNING
            match state_clone.compare_exchange(
                IDLE,
                RUNNING,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    println!("Worker {}: Job started!", i);
                    thread::sleep(std::time::Duration::from_secs(1));
                    // Transition to FINISHED
                    state_clone.store(FINISHED, Ordering::SeqCst);
                    println!("Worker {}: Job finished.", i);
                }
                Err(current_state) => {
                    println!(
                        "Worker {}: Could not start job, state is already {}.",
                        i, current_state
                    );
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("Final state: {}", state.load(Ordering::SeqCst));
}