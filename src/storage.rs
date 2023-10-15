use std::{path::PathBuf, fs};

use crate::cdn::rest::Resource;

#[derive(Clone)]
pub struct Storage {
    storage_path: String,
}

impl Storage {
    pub fn new(storage_path: String) -> Self {
        Self {
            storage_path,
        }
    }

    pub fn get(
        &self,
        resource: Resource,
        id: &str,
        hash: &str,
    ) -> Option<Vec<u8>> {
        let path = PathBuf::new()
            .join(&self.storage_path)
            .join(resource.to_string())
            .join(id)
            .join(hash);

        match path.try_exists() {
            Ok(true) => {
                Some(fs::read(path).unwrap_or_default())
            },
            _ => None
        }
    }

    pub fn _put(&self, resource: Resource, id: &str, image_data: Vec<u8>) {
        let digest = md5::compute(image_data);
        let hash = format!("{digest:x}");

        println!("hash: {hash}");
    }
}
