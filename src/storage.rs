use std::fs::OpenOptions;
use std::{fs, path::PathBuf};

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
    pub fn new(storage_path: &str) -> Self {
        Self {
            storage_path: storage_path.to_string(),
        }
    }

    fn path(&self, resource: &Resource, id: &str) -> PathBuf {
        PathBuf::new()
            .join(&self.storage_path)
            .join(resource.to_string())
            .join(id)
    }

    pub fn get(&self, resource: Resource, id: &str, filename: &str) -> Option<Vec<u8>> {
        let path = self.path(&resource, id).join(filename);

        match path.try_exists() {
            Ok(true) => fs::read(path).ok(),
            _ => None,
        }
    }

    pub fn put(
        &self,
        resource: Resource,
        id: &str,
        image_data: Vec<u8>,
        hash: &str,
    ) -> Result<String> {
        let reader = Reader::new(Cursor::new(&image_data)).with_guessed_format()?;

        let format = reader
            .format()
            .ok_or_else(|| anyhow!("Invalid file format"))?;

        match format {
            ImageFormat::Gif | ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP => (),
            _ => return Err(anyhow!("Unsupported image format")),
        };

        let is_animated = format.eq(&ImageFormat::Gif);

        let filename = if !is_animated {
            format!("{hash}.png")
        } else {
            format!("a_{hash}.png")
        };

        let base_path = self.path(&resource, id);

        let image = reader.decode()?;
        let default_image_size = 1024;

        let smallest_dimension = std::cmp::min(image.width(), image.height());
        let image_size = std::cmp::min(smallest_dimension, default_image_size);
        let image = image.crop_imm(0, 0, image_size, image_size);

        fs::create_dir_all(&base_path)?;

        if resource.singleton() {
            for entry in base_path.read_dir()? {
                fs::remove_file(entry?.path())?;
            }
        }

        let base_path = base_path.join(&filename);

        let dest_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(base_path)?;

        let png_options =
            PngEncoder::new_with_quality(dest_file, CompressionType::Fast, FilterType::NoFilter);

        image.write_with_encoder(png_options)?;

        Ok(filename)
    }
}
