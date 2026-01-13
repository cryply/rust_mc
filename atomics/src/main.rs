use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..1000 {
                // Atomically adds 1 and returns the previous value.
                // SeqCst is the strongest ordering, ensuring a total order.
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // The final value is guaranteed to be 10 * 1000 = 10000
    println!("Final counter value: {}", counter.load(Ordering::SeqCst));
}