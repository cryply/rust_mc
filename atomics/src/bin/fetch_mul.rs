// This requires a recent Rust compiler (1.60+)
use std::sync::atomic::{AtomicUsize, Ordering, AtomicU64};
use std::sync::Arc;
use std::thread;



// Change from this:
//   static mut COUNTER: u64 = 0;
// to this:
static COUNTER: AtomicU64 = AtomicU64::new(0);


fn main() {


    COUNTER.fetch_add(1, Ordering::Relaxed);



    let value = Arc::new(AtomicUsize::new(2));

    let mut handles = vec![];
    for i in 0..4 {
        let value_clone = Arc::clone(&value);
        let handle = thread::spawn(move || {
            // Each thread multiplies the current value by its ID + 2
            let multiplier = i + 2;
            println!("Thread {} multiplying by {}", i, multiplier);
            value_clone.fetch_mul(multiplier, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // The final value is 2 * 2 * 3 * 4 * 5 = 240
    println!("Final value: {}", value.load(Ordering::SeqCst));
}