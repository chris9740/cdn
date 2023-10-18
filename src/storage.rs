use std::fs::OpenOptions;
use std::{env, fs, path::PathBuf};

use anyhow::{anyhow, Result};
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::io::Reader;
use image::ImageFormat;
use std::io::Cursor;

use crate::rest::Resource;

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

        match path.try_exists() {
            Ok(true) => Some(fs::read(path).unwrap_or_default()),
            _ => None,
        }
    }

    pub fn put(&self, resource: Resource, id: &str, image_data: Vec<u8>) -> Result<String> {
        let digest = md5::compute(&image_data);
        let hash = format!("{digest:x}");
        let filename = format!("{hash}.png");
        let current_dir = env::current_dir()?;
        let target_path = current_dir.join(self.build_path(resource, id, &filename));
        let reader = Reader::new(Cursor::new(&image_data)).with_guessed_format()?;

        match reader.format() {
            Some(ImageFormat::Gif | ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP) => (),
            Some(_) => return Err(anyhow!("Unsupported image format")),
            _ => return Err(anyhow!("Invalid file format")),
        };

        let image = reader.decode()?;
        let default_image_size = 1024;

        let smallest_dimension = std::cmp::min(image.width(), image.height());
        let image_size = std::cmp::min(smallest_dimension, default_image_size);

        let image = image.crop_imm(0, 0, image_size, image_size);

        match target_path.parent() {
            Some(parent) => fs::create_dir_all(parent)?,
            None => return Err(anyhow!("Could not get parent directory for target path")),
        };

        let file = OpenOptions::new().write(true).create(true).open(target_path)?;
        let png_options =
            PngEncoder::new_with_quality(file, CompressionType::Fast, FilterType::Adaptive);

        image.write_with_encoder(png_options)?;

        Ok(hash)
    }
}
