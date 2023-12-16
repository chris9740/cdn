use std::{
    io::{BufReader, Cursor},
    sync::Arc,
};

use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError},
    web, HttpRequest, HttpResponse, Result,
};
use image::{imageops::FilterType, io::Reader, ImageFormat};
use serde::Deserialize;

use crate::{cdn::{Cdn, Connected}, unwrap_or_return};

use super::{GenericError, Resource};

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    size: Option<u32>,
}

const MAX_SIZE: u32 = 2048;
const SIZES: [u32; 5] = [128, 256, 512, 1024, MAX_SIZE];

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

    let size = query.size.unwrap_or(256);
    let resource_type = Resource::try_from(segments[0]);

    if !SIZES.contains(&size) {
        return Err(ErrorBadRequest("The specified size is not a valid number"));
    }

    if size > MAX_SIZE {
        return Err(ErrorBadRequest(format!("Size cannot be larger than {MAX_SIZE}")));
    }

    if let Ok(resource) = resource_type {
        let id = &path.0;
        let image_hash = &path.1;
        let ext = &path.2;
        let filename = format!("{image_hash}.{ext}");

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
            _ => cdn.storage.get(resource, id, &filename),
        };

        return match image_data {
            Some(image_data) => {
                let bytes = if is_from_cache {
                    image_data
                } else {
                    let cursor = Cursor::new(&image_data);
                    let buf_reader = BufReader::new(cursor);
                    let mut reader = Reader::new(buf_reader);

                    reader.set_format(ImageFormat::Png);

                    let mut image = unwrap_or_return!(
                        reader.decode(),
                        ErrorInternalServerError("Decoding of image failed")
                    );

                    image = image.resize_exact(size, size, FilterType::Triangle);

                    let mut buffer = Cursor::new(Vec::new());

                    unwrap_or_return!(
                        image.write_to(&mut buffer, ImageFormat::Png),
                        ErrorInternalServerError("Buffer overflow")
                    );

                    let bytes = buffer.into_inner();

                    unwrap_or_return!(
                        cdn.cache.put(&mut con, &key, &bytes),
                        ErrorInternalServerError("Failed to write to cache")
                    );

                    bytes
                };

                Ok(HttpResponse::Ok()
                    .content_type("image/png")
                    .append_header((
                        "X-Origin-Status",
                        if is_from_cache { "cache" } else { "origin" },
                    ))
                    .body(bytes))
            }
            None => Ok(HttpResponse::NotFound().json(GenericError {
                error: "Image not found".to_string(),
            })),
        };
    }

    Ok(HttpResponse::NotFound().json(GenericError {
        error: "Resource not found".to_string(),
    }))
}
