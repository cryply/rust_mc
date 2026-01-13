
use std::{ptr, slice};
use std::sync::Arc;
struct SafeWrapper {
    ptr: *const u8,
    len: usize,
}

/*
Safe only if pointer point to heap allocated data that outlives thread (Box, Vec, Arc)
And you can manually implement:

unsafe impl<T: Send> Send for *const T {}
unsafe impl<T: Send> Send for *mut T {}

unsafe impl<T: Sync> Sync for *const T {}
unsafe impl<T: Sync> Sync for *mut T {}

*/


// We know this pointer is never mutated, so it's safe to share!
unsafe impl Sync for SafeWrapper {}

// We can safely move the pointer between threads.
unsafe impl Send for SafeWrapper {}

fn main() {

    let s = String::from("Hello!");

    let x: SafeWrapper = SafeWrapper { 
        ptr: s.as_ptr(),
        len: s.len(),
    };
    let a: Arc<SafeWrapper> = Arc::new(x);
    let cloned_a = Arc::clone(&a);

    let h = std::thread::spawn(move || {
        let bytes = unsafe{
            slice::from_raw_parts(cloned_a.ptr, cloned_a.len)
        };
        println!("{}", String::from_utf8_lossy(bytes.as_ref()) );
    });

    h.join();
}
