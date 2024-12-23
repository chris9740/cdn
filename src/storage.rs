use std::fs::OpenOptions;
use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};
use image::codecs::gif::{GifEncoder, Repeat};
use image::codecs::png::PngEncoder;
use image::{codecs::gif::GifDecoder, io::Reader, DynamicImage, ImageOutputFormat::Png};
use image::{AnimationDecoder, Frame, GenericImageView, ImageEncoder, ImageFormat, RgbaImage};
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

        let base_path = self.path(&resource, id);
        fs::create_dir_all(&base_path)?;

        match format {
            ImageFormat::Gif => {
                let decoder = GifDecoder::new(Cursor::new(&image_data))?;
                let frames = decoder.into_frames();
                let mut cropped_frames = Vec::new();

                let mut first_frame_png: Option<RgbaImage> = None;

                for frame in frames.collect_frames()? {
                    let buffer = frame.clone().into_buffer();
                    let dynamic_image = DynamicImage::ImageRgba8(buffer);
                    let cropped_image = crop_to_square(&dynamic_image);

                    if first_frame_png.is_none() {
                        first_frame_png = Some(cropped_image.to_rgba8());
                    }

                    cropped_frames.push(Frame::from_parts(
                        cropped_image.to_rgba8(),
                        0,
                        0,
                        frame.delay(),
                    ));
                }

                if resource.singleton() {
                    for entry in fs::read_dir(&base_path)? {
                        let entry = entry?;
                        let path = entry.path();

                        if path.is_file() {
                            fs::remove_file(path)
                                .map_err(|err| anyhow!("Failed to remove file: {err}"))?;
                        }
                    }
                }

                let png_filename = format!("a_{hash}.png");
                let gif_filename = format!("a_{hash}.gif");

                let png_path = base_path.join(&png_filename);
                let gif_path = base_path.join(gif_filename);

                // We want to show a still image until hover
                if let Some(first_frame) = first_frame_png {
                    let mut png_file = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(png_path)?;

                    PngEncoder::new(&mut png_file).write_image(
                        first_frame.as_raw(),
                        first_frame.width(),
                        first_frame.height(),
                        image::ColorType::Rgba8,
                    )?;
                }

                let gif_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(gif_path)?;

                println!("Using speed 30");
                let mut gif_encoder = GifEncoder::new_with_speed(gif_file, 30);
                gif_encoder.set_repeat(Repeat::Infinite)?;

                println!("Total frames: {}", cropped_frames.len());

                gif_encoder.encode_frames(cropped_frames)?;

                Ok(png_filename)
            }
            ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP => {
                let filename = format!("{hash}.png");
                let path = base_path.join(&filename);
                let image = reader.decode()?;
                let cropped_image = crop_to_square(&image);

                if resource.singleton() {
                    for entry in fs::read_dir(&base_path)? {
                        let entry = entry?;
                        let path = entry.path();

                        if path.is_file() {
                            fs::remove_file(path)
                                .map_err(|err| anyhow!("Failed to remove file: {err}"))?;
                        }
                    }
                }

                let mut dest_file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(path)
                    .map_err(|err| anyhow!("Failed to open file: {err}"))?;
                cropped_image
                    .write_to(&mut dest_file, Png)
                    .map_err(|err| anyhow!("Failed to write image: {err}"))?;

                Ok(filename)
            }
            _ => Err(anyhow!("Unsupported image format")),
        }
    }
}

fn crop_to_square(image: &DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();

    let crop_size = std::cmp::min(width, height);

    let top = (height - crop_size) / 2;
    let left = (width - crop_size) / 2;

    image.crop_imm(left, top, crop_size, crop_size)
}
