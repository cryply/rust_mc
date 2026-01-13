use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

struct MyArc<T> {
    // The reference count and the data are often stored together
    // in a separate allocation, but for simplicity, we'll separate them.
    data: *const T,
    ref_count: AtomicUsize,
}

// NOTE: This is a simplified and incomplete example.
// A real implementation would need to handle deallocation correctly
// and be thread-safe in all aspects (e.g., using `unsafe` correctly).
// This is just to demonstrate the atomic operations.

impl<T> MyArc<T> {
    fn new(data: T) -> Self {
        let boxed = Box::new(data);
        MyArc {
            data: Box::into_raw(boxed),
            ref_count: AtomicUsize::new(1),
        }
    }

    fn clone(&self) -> Self {
        // Atomically increment the reference count.
        // Use AcqRel to prevent reordering around this operation.
        self.ref_count.fetch_add(1, Ordering::AcqRel);
        MyArc {
            data: self.data,
            ref_count: AtomicUsize::new(self.ref_count.load(Ordering::Relaxed)),
        }
    }
}

fn main() {
    let my_data = "I am shared data".to_string();
    let arc1 = MyArc::new(my_data);

    let mut handles = vec![];
    for i in 0..5 {
        let arc_clone = arc1.clone();
        let handle = thread::spawn(move || {
            // In a real `Arc`, accessing data would be safe.
            // Here we'd need `unsafe`.
            println!("Thread {} is using the data.", i);
            // `arc_clone` goes out of scope here, and `drop` would be called,
            // which would call `fetch_sub`.
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Initial ref count was 1, now it should be 6.");
    // The final ref count would be checked before deallocation.
    // For this example, we'll leak the memory for simplicity.
}
