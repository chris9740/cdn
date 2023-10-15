// cache.rs

use std::collections::HashMap;
use std::sync::{RwLock, Mutex, Arc};

#[derive(Clone)]
pub struct Cache {
    data: Arc<RwLock<HashMap<String, Mutex<Vec<u8>>>>>
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(mutex) = self.data.read().unwrap().get(key) {
            let data = mutex.lock().unwrap();
            Some(data.clone())
        } else {
            None
        }
    }

    pub fn put(&self, key: String, value: Vec<u8>) {
        let mut cache = self.data.write().unwrap();
        let mutex = cache.entry(key).or_insert(Mutex::new(value.clone()));
        let mut data = mutex.lock().unwrap();
        *data = value;
    }
}
