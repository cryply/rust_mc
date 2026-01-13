use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const NO_TOKEN: usize = 0;

fn main() {
    let token_slot = Arc::new(AtomicUsize::new(NO_TOKEN));

    // Producer thread
    let producer = {
        let token_slot = Arc::clone(&token_slot);
        thread::spawn(move || {
            let token = 123; // The "job" or token
            println!("Producer: trying to place token {}", token);
            token_slot.swap(token, Ordering::SeqCst);
            println!("Producer: token placed.");
        })
    };

    // Consumer thread
    let consumer = {
        let token_slot = Arc::clone(&token_slot);
        thread::spawn(move || {
            println!("Consumer: waiting for token...");
            let mut token;
            loop {
                // Swap the current value with NO_TOKEN.
                // If there was a token, we get it back.
                // If not, we get NO_TOKEN back and continue looping.
                token = token_slot.swap(NO_TOKEN, Ordering::SeqCst);
                if token != NO_TOKEN {
                    break;
                }
                thread::sleep(Duration::from_millis(100));
            }
            println!("Consumer: got token {}!", token);
        })
    };

    producer.join().unwrap();
    consumer.join().unwrap();
}