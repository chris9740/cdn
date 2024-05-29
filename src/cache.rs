use anyhow::Result;
use redis::{Commands, Connection, RedisResult};
use serde::Serialize;

#[derive(Clone)]
pub struct Cache;

#[derive(Debug, Serialize)]
pub struct Health {
    memory_usage: String,
    keys: u32,
    uptime_seconds: u32,
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    pub fn new() -> Self {
        Cache {}
    }

    pub fn get(&self, con: &mut Connection, key: &str) -> Option<Vec<u8>> {
        let data: RedisResult<Vec<u8>> = con.get(key);

        match data {
            Ok(vec) if !vec.is_empty() => Some(vec),
            _ => None,
        }
    }

    pub fn put(&self, con: &mut Connection, key: &str, value: &Vec<u8>) -> Result<()> {
        con.set(key, value)?;
        con.expire(key, 60 * 5)?;

        Ok(())
    }

    pub fn get_redis_health(&self, mut con: &mut Connection) -> Result<Health> {
        let (info, keys): (String, u32) =
            redis::pipe().cmd("INFO").cmd("DBSIZE").query(&mut con)?;

        fn get_info_key(info: &str, key: &str) -> Option<String> {
            info.lines()
                .find(|line| line.starts_with(key))
                .map(|line| line.trim_start_matches(key).to_string())
        }

        let memory_usage =
            get_info_key(&info, "used_memory_human:").unwrap_or_else(|| "(error)".to_string());

        let uptime_seconds = get_info_key(&info, "uptime_in_seconds:")
            .map(|uptime| uptime.parse().unwrap_or(0))
            .unwrap_or(0);

        Ok(Health {
            memory_usage,
            keys,
            uptime_seconds,
        })
    }
}
