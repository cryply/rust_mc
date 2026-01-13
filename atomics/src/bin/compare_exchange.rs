use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

// A mock expensive computation
fn expensive_computation() -> usize {
    println!("Performing expensive computation...");
    thread::sleep(std::time::Duration::from_secs(1));
    42
}

fn main() {
    // 0 means "not initialized yet". Any other value is the result.
    let cached_value = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];
    for _ in 0..5 {
        let cached_value_clone = Arc::clone(&cached_value);
        let handle = thread::spawn(move || {
            // Load the current value.
            let mut current_value = cached_value_clone.load(Ordering::Acquire);

            if current_value == 0 {
                // Try to compute and set the value.
                let new_value = expensive_computation();

                // CAS loop: keep trying until we succeed or another thread succeeds.
                loop {
                    match cached_value_clone.compare_exchange_weak(
                        current_value,
                        new_value,
                        Ordering::Release, // Success ordering
                        Ordering::Relaxed, // Failure ordering
                    ) {
                        Ok(_) => {
                            // We successfully set the value.
                            println!("Thread initialized the value.");
                            break;
                        }
                        Err(actual) => {
                            // Another thread set the value before we could.
                            // Use the value that the other thread set.
                            current_value = actual;
                            break;
                        }
                    }
                }
            }
            println!("Thread got value: {}", current_value);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}