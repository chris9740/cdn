use crate::{cache::Cache, storage::Storage};

#[derive(Clone)]
pub struct Cdn {
    pub storage: Storage,
    pub cache: Cache,
}

impl Cdn {
    pub fn new(storage: Storage, cache: Cache) -> Self {
        Self { storage, cache }
    }
}
