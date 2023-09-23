use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct RedisValueStore {
    state: Arc<Mutex<HashMap<String, String>>>,
}

impl RedisValueStore {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.state.lock().unwrap().insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.state.lock().unwrap().get(key).cloned()
    }
}
