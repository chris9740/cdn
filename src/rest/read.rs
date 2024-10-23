use std::{
    io::{BufReader, Cursor},
    sync::Arc,
};

use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError},
    web, HttpRequest, HttpResponse, Result,
};
use image::{
    codecs::gif::{GifDecoder, GifEncoder, Repeat},
    codecs::png::PngDecoder,
    imageops::FilterType,
    AnimationDecoder, DynamicImage, Frame, ImageOutputFormat,
};
use serde::Deserialize;

use crate::{
    cdn::{Cdn, Connected},
    unwrap_or_return,
};

use super::{GenericError, Resource};

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    size: Option<u32>,
}

enum ImageFormat {
    Png,
    Gif,
}

impl ImageFormat {
    fn content_type(&self) -> &str {
        match self {
            Self::Gif => "image/gif",
            Self::Png => "image/png",
        }
    }

    fn max_size(&self) -> u32 {
        match self {
            Self::Gif => MAX_SIZE_GIF,
            Self::Png => MAX_SIZE_PNG,
        }
    }
}

impl TryFrom<&str> for ImageFormat {
    type Error = String;

    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        match value {
            "gif" => Ok(Self::Gif),
            "png" => Ok(Self::Png),
            _ => Err(String::from("Unknown image format")),
        }
    }
}

const MAX_SIZE_PNG: u32 = 2048;
const DEFAULT_SIZE: u32 = 256;
const MAX_SIZE_GIF: u32 = DEFAULT_SIZE;
const SIZES: [u32; 5] = [128, DEFAULT_SIZE, 512, 1024, MAX_SIZE_PNG];

pub async fn get_resource(
    request: HttpRequest,
    path: web::Path<(String, String, String)>,
    data: web::Data<Arc<Cdn<Connected>>>,
    query: web::Query<QueryParams>,
) -> Result<HttpResponse> {
    let uri = request.uri().to_string();
    let segments: Vec<&str> = uri
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    let size = query.size.unwrap_or(DEFAULT_SIZE);
    let resource_type = Resource::try_from(segments[0]);

    if !SIZES.contains(&size) {
        return Err(ErrorBadRequest("The specified size is not valid"));
    }

    if let Ok(resource) = resource_type {
        let id = &path.0;
        let image_hash = &path.1;
        let ext = &path.2;
        let filename = format!("{image_hash}.{ext}");
        let image_format = unwrap_or_return!(
            ImageFormat::try_from(ext.as_str()),
            ErrorBadRequest("Invalid image extension")
        );
        let max_size = image_format.max_size();

        if size > max_size {
            return Err(ErrorBadRequest(format!(
                "Size of a {ext} image cannot be larger than {max_size}"
            )));
        }

        let key = format!("{id}:{image_hash}:{ext}:{size}");
        let cdn = data.get_ref();

        let mut is_from_cache = false;
        let redis = data.redis();

        let mut con = unwrap_or_return!(
            redis.lock(),
            ErrorInternalServerError("Connection error with redis")
        );

        let image_data = match cdn.cache.get(&mut con, &key) {
            Some(data) => {
                is_from_cache = true;
                Some(data)
            }
            None => cdn.storage.get(resource, id, &filename),
        };

        return match image_data {
            Some(image_data) => {
                let bytes = if is_from_cache {
                    image_data
                } else {
                    let cursor = Cursor::new(&image_data);
                    let buf_reader = BufReader::new(cursor);

                    match image_format {
                        ImageFormat::Gif => {
                            let decoder = unwrap_or_return!(
                                GifDecoder::new(buf_reader),
                                ErrorBadRequest("Failed to create GIF decoder")
                            );
                            let frames = decoder.into_frames();
                            let frames = unwrap_or_return!(
                                frames.collect_frames(),
                                ErrorBadRequest("Error collecting frames")
                            );

                            let mut output_frames = Vec::new();
                            for frame in frames {
                                let buffer = frame.clone().into_buffer();
                                let mut image = DynamicImage::ImageRgba8(buffer.clone());
                                image = image.resize_exact(size, size, FilterType::Triangle);
                                output_frames.push(Frame::from_parts(
                                    image.to_rgba8(),
                                    0,
                                    0,
                                    frame.delay(),
                                ));
                            }

                            let mut buffer = Vec::new();
                            {
                                let mut gif_encoder = GifEncoder::new_with_speed(&mut buffer, 30);
                                unwrap_or_return!(
                                    gif_encoder.set_repeat(Repeat::Infinite),
                                    ErrorBadRequest("Error encoding frames")
                                );
                                unwrap_or_return!(
                                    gif_encoder.encode_frames(output_frames),
                                    ErrorBadRequest("Error encoding frames")
                                );
                            }

                            unwrap_or_return!(
                                cdn.cache.put(&mut con, &key, &buffer),
                                ErrorInternalServerError("Failed to write to cache")
                            );

                            buffer
                        }
                        ImageFormat::Png => {
                            let decoder = unwrap_or_return!(
                                PngDecoder::new(buf_reader),
                                ErrorBadRequest("Failed to create PNG decoder")
                            );
                            let mut image = unwrap_or_return!(
                                DynamicImage::from_decoder(decoder),
                                ErrorBadRequest("Failed to decode PNG")
                            );

                            image = image.resize_exact(size, size, FilterType::Triangle);

                            let mut buffer = Vec::new();
                            unwrap_or_return!(
                                image.write_to(
                                    &mut Cursor::new(&mut buffer),
                                    ImageOutputFormat::Png
                                ),
                                ErrorInternalServerError("Failed to write PNG to buffer")
                            );

                            unwrap_or_return!(
                                cdn.cache.put(&mut con, &key, &buffer),
                                ErrorInternalServerError("Failed to write to cache")
                            );

                            buffer
                        }
                    }
                };

                let content_type = image_format.content_type();

                Ok(HttpResponse::Ok()
                    .content_type(content_type)
                    .append_header((
                        "X-Origin-Status",
                        if is_from_cache { "cache" } else { "origin" },
                    ))
                    .body(bytes))
            }
            None => Ok(HttpResponse::NotFound().finish()),
        };
    }

    Ok(HttpResponse::NotFound().json(GenericError {
        error: "Resource not found".to_string(),
    }))
}
