use crate::{cache::Cache, storage::Storage, macros::error};

pub struct Cdn {
    pub storage: Storage,
    pub cache: Cache,
    pub redis: redis::Client,
}

impl Cdn {
    pub fn new(storage: Storage, cache: Cache) -> Self {
        let redis =
            redis::Client::open("redis://127.0.0.1").unwrap_or_else(|why| {
                error!("Could not connect to redis: {}", why.to_string());
            });

        Self {
            storage,
            cache,
            redis,
        }
    }
}
