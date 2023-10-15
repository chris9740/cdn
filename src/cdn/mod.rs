use crate::{cache::Cache, storage::Storage};

pub mod rest;

#[derive(Clone)]
pub struct Cdn {
    storage: Storage,
    cache: Cache,
}

impl Cdn {
    pub fn new(storage: Storage, cache: Cache) -> Self {
        storage._put(rest::Resource::Avatars, "id", vec![1, 2, 3]);

        Self { storage, cache }
    }
}
