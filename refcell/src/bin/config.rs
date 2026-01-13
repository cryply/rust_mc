use std::cell::RefCell;
use std::collections::HashMap;

struct Config {
    settings: RefCell<HashMap<String, String>>,
}

impl Config {
    fn new() -> Self {
        Config {
            settings: RefCell::new(HashMap::new()),
        }
    }

    fn set(&self, key: &str, value: &str) {
        self.settings.borrow_mut().insert(key.to_string(), value.to_string());
    }

    fn get(&self, key: &str) -> Option<String> {
        self.settings.borrow().get(key).cloned()
    }
}

fn main() {
    let cfg = Config::new();
    cfg.set("theme", "dark");
    println!("Theme: {:?}", cfg.get("theme")); // Some("dark")
}