use crate::{cache::Cache, storage::Storage};

pub mod rest;

#[derive(Clone)]
pub struct CDN {
    storage: Storage,
    cache: Cache,
}

impl CDN {
    pub fn new(storage: Storage, cache: Cache) -> Self {
        storage._put(rest::Resource::Avatars, "id", vec![1, 2, 3]);

        Self { storage, cache }
    }
}
