use anyhow::Result;
use serde::Serialize;
use redis::Commands;

#[derive(Clone)]
pub struct Cache {
    pub redis_client: redis::Client,
}

#[derive(Debug, Serialize)]
pub struct Health {
    memory_usage: String,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            redis_client: redis::Client::open("redis://127.0.0.1")
                .expect("Failed to open connection to redis"),
        }
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut conn = self.redis_client.get_connection().ok()?;
        let data: redis::RedisResult<Option<Vec<u8>>> = conn.get(key);

        data.ok()?
    }

    pub fn put(&self, key: String, value: &Vec<u8>) -> Result<()> {
        let mut conn = self.redis_client.get_connection()?;

        conn.set(key, value)?;

        Ok(())
    }

    pub fn health(&self) -> Option<Health> {
        let mut conn = self.redis_client.get_connection().ok()?;
        let info: String = redis::cmd("INFO").query(&mut conn).ok()?;

        let memory_usage: String = info
            .lines()
            .find(|line| line.starts_with("used_memory_human:"))
            .map(|line| line.trim_start_matches("used_memory_human:").to_string())
            .unwrap_or("(error)".to_string());

        Some(Health { memory_usage })
    }
}
