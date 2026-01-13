use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub struct SpinLock {
    locked: AtomicBool,
}

impl SpinLock {
    pub fn new() -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) {
        // Keep trying to swap `false` to `true`.
        while self
            .locked
            .compare_exchange_weak(
                false, // Current value we expect
                true,  // New value we want to set
                Ordering::Acquire, // On success, acquire lock
                Ordering::Relaxed, // On failure, just spin
            )
            .is_err()
        {
            // Hint to the CPU that we're in a busy-wait loop.
            std::hint::spin_loop();
        }
    }

    pub fn unlock(&self) {
        // Release the lock by setting it to false.
        self.locked.store(false, Ordering::Release);
    }
}

fn main() {
    let lock = Arc::new(SpinLock::new());
    let mut handles = vec![];

    for i in 0..5 {
        let lock_clone = Arc::clone(&lock);
        let handle = thread::spawn(move || {
            lock_clone.lock();
            println!("Thread {} has the lock", i);
            // Simulate work
            thread::sleep(std::time::Duration::from_millis(100));
            println!("Thread {} is releasing the lock", i);
            lock_clone.unlock();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}