use std::{env, fs, path::PathBuf};

use anyhow::{anyhow, Result};
use image::io::Reader;
use image::ImageFormat;
use std::io::Cursor;

use crate::cdn::rest::Resource;

#[derive(Clone)]
pub struct Storage {
    storage_path: String,
}

impl Storage {
    pub fn new(storage_path: String) -> Self {
        Self { storage_path }
    }

    fn build_path(&self, resource: Resource, id: &str, filename: &str) -> PathBuf {
        PathBuf::new()
            .join(&self.storage_path)
            .join(resource.to_string())
            .join(id)
            .join(filename)
    }

    pub fn get(&self, resource: Resource, id: &str, filename: &str) -> Option<Vec<u8>> {
        let path = self.build_path(resource, id, filename);

        println!("{path:?}");

        match path.try_exists() {
            Ok(true) => Some(fs::read(path).unwrap_or_default()),
            _ => None,
        }
    }

    pub fn put(&self, resource: Resource, id: &str, image_data: Vec<u8>) -> Result<()> {
        let digest = md5::compute(&image_data);
        let hash = format!("{digest:x}");
        let filename = format!("{hash}.png");

        let current_dir = env::current_dir()?;
        let target_path = current_dir.join(self.build_path(resource, id, &filename));

        let reader = Reader::new(Cursor::new(&image_data))
            .with_guessed_format()
            .expect("Couldn't get format");

        match reader.format() {
            Some(ImageFormat::Gif | ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP) => (),
            Some(_) => return Err(anyhow!("Unsupported image format")),
            _ => return Err(anyhow!("Invalid file format")),
        };

        let image = reader.decode()?;

        let smallest_dimension = std::cmp::min(image.width(), image.height());
        let final_size = std::cmp::min(smallest_dimension, 1024);

        let image = image.crop_imm(0, 0, final_size, final_size);

        match target_path.parent() {
            Some(parent) => fs::create_dir_all(parent)?,
            None => return Err(anyhow!("Could not get parent directory for target path")),
        };

        image.save(target_path)?;

        Ok(())
    }
}
