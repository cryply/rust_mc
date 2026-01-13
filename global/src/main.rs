use std::collections::{ HashSet};
use std::sync::{Mutex, OnceLock};


static CACHE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn main() {

    CACHE.get_or_init(|| Mutex::new(HashSet::new()));


    let h = std::thread::spawn(|| {
        CACHE.get().unwrap().lock().unwrap().insert("value".to_string());
    });

    h.join().unwrap();

    println!("Got {}", CACHE.get().unwrap().lock().unwrap().contains("value1"));
}