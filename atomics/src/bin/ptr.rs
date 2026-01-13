use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    let data = "Hello, atomic pointer!".to_owned();
    let initial_ptr = Box::into_raw(Box::new(data));

    // AtomicPtr holds a raw pointer to our string.
    let atomic_ptr = Arc::new(AtomicPtr::new(initial_ptr));

    let atomic_ptr_clone = Arc::clone(&atomic_ptr);
    let handle = thread::spawn(move || {
        // Atomically load the pointer.
        let ptr = atomic_ptr_clone.load(Ordering::Acquire);

        // SAFETY: The main thread ensures the pointer is valid.
        // We are only reading the data here.
        let s = unsafe { &*ptr };
        println!("Thread 1 reads: {}", s);
    });

    // Main thread can also read the pointer.
    let ptr = atomic_ptr.load(Ordering::Acquire);
    let s = unsafe { &*ptr };
    println!("Main thread reads: {}", s);

    handle.join().unwrap();

    // Remember to deallocate the memory to avoid a leak.
    let ptr = atomic_ptr.swap(std::ptr::null_mut(), Ordering::AcqRel);
    unsafe { drop(Box::from_raw(ptr)) };
}