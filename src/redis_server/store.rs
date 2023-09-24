use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct StoreValue {
    pub data: String,
    pub timeout: Option<Instant>,
}


#[derive(Clone)]
pub struct RedisValueStore {
    state: HashMap<String, StoreValue>,
}

impl RedisValueStore {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }

    pub fn set_with_expiry(&mut self, key: String, value: String, t: u64) {
        self.state
            .insert(key, StoreValue {
                data: value,
                timeout: Some(Instant::now() + Duration::from_millis(t)),
            });
    }

    pub fn set(&mut self, key: String, value: String) {
        self.state.insert(key, StoreValue { data: value, timeout: None});
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        match self.state.get(key) {
            Some(entry) => {
                if let Some(t) = &entry.timeout {
                    if Instant::now() > t.clone() {
                        self.state.remove(key);
                        return None;
                    }
                }
                Some(entry.data.clone())
            }
            None => None,
        }
    }
}
