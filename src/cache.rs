// cache.rs

use std::collections::HashMap;
use std::sync::{RwLock, Mutex, Arc};

use anyhow::{Result, anyhow};

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
        if let Some(mutex) = self.data.read().ok()?.get(key) {
            let data = mutex.lock().ok()?;
            Some(data.clone())
        } else {
            None
        }
    }

    pub fn put(&self, key: String, value: Vec<u8>) -> Result<()> {
        let mut cache = self.data.write().or(Err(anyhow!("RwLock poisoned")))?;

        let mutex = cache.entry(key).or_insert(Mutex::new(value.clone()));
        let data = mutex.get_mut().or(Err(anyhow!("Failed to get mutable reference to data")))?;

        *data = value;

        Ok(())
    }
}
