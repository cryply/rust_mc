use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

const FLAG_A: usize = 1 << 0; // Binary ...0001
const FLAG_B: usize = 1 << 1; // Binary ...0010
const FLAG_C: usize = 1 << 2; // Binary ...0100

fn main() {
    let flags = AtomicUsize::new(0);

    // Thread 1 sets FLAG_A
    thread::spawn(move || {
        println!("Thread 1 setting FLAG_A");
        flags.fetch_or(FLAG_A, Ordering::Relaxed);
    });

    // Thread 2 sets FLAG_B
    thread::spawn(move || {
        println!("Thread 2 setting FLAG_B");
        flags.fetch_or(FLAG_B, Ordering::Relaxed);
    });

    thread::sleep(std::time::Duration::from_millis(10));

    let current_flags = flags.load(Ordering::SeqCst);
    println!("Current flags: {:b}", current_flags);
    println!("FLAG_A is set: {}", (current_flags & FLAG_A) != 0);
    println!("FLAG_B is set: {}", (current_flags & FLAG_B) != 0);
    println!("FLAG_C is set: {}", (current_flags & FLAG_C) != 0);

    // Clear FLAG_A
    println!("Clearing FLAG_A");
    flags.fetch_and(!FLAG_A, Ordering::Relaxed);

    let final_flags = flags.load(Ordering::SeqCst);
    println!("Final flags: {:b}", final_flags);
    println!("FLAG_A is set: {}", (final_flags & FLAG_A) != 0);
}