use std::cell::RefCell;
use std::collections::HashMap;

struct Cache {
    data: RefCell<HashMap<String, String>>,
}

impl Cache {
    fn new() -> Self {
        Cache { data: RefCell::new(HashMap::new()) }
    }

    fn get(&self, key: &str) -> String {
        // Try to get value immutably
        if let Some(val) = self.data.borrow().get(key) {
            return val.clone();
        }

        // Value missing → generate & insert (need mutable access)
        let new_val = format!("Generated for '{}'", key);
        self.data.borrow_mut().insert(key.to_string(), new_val.clone());
        new_val
    }
}

fn main() {
    let cache = Cache::new();
    println!("{}", cache.get("foo")); // Generated for 'foo'
    println!("{}", cache.get("foo")); // (cached) Generated for 'foo'
}