use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    // A flag to signal shutdown.
    let shutdown = Arc::new(AtomicBool::new(false));

    let worker_handle = {
        let shutdown = Arc::clone(&shutdown);
        thread::spawn(move || {
            println!("Worker thread started.");
            while !shutdown.load(Ordering::Relaxed) {
                // Do some work...
                println!("Working...");
                thread::sleep(Duration::from_millis(500));
            }
            println!("Worker thread shutting down.");
        })
    };

    thread::sleep(Duration::from_secs(2));
    println!("Main thread sending shutdown signal.");
    // Set the flag to true.
    shutdown.store(true, Ordering::Relaxed);

    worker_handle.join().unwrap();
    println!("Main thread finished.");
}