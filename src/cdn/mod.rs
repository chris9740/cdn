use crate::{cache::Cache, storage::Storage};

pub mod rest;

#[derive(Clone)]
pub struct Cdn {
    storage: Storage,
    cache: Cache,
}

impl Cdn {
    pub fn new(storage: Storage, cache: Cache) -> Self {
        Self { storage, cache }
    }
}
