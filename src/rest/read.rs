use std::{io::Cursor, sync::Arc};

use actix_web::{error::ErrorInternalServerError, web, HttpRequest, HttpResponse, Result};
use image::{imageops::FilterType, io::Reader, ImageFormat};
use serde::Deserialize;

use crate::cdn::Cdn;

use super::Resource;

macro_rules! unwrap_or_return {
    ($result:expr, $error:expr) => {
        match $result {
            Ok(val) => val,
            Err(_) => {
                return Err($error);
            }
        }
    };
}

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    size: Option<u32>,
}

pub async fn get_resource(
    request: HttpRequest,
    path: web::Path<(String, String)>,
    data: web::Data<Arc<Cdn>>,
    query: web::Query<QueryParams>,
) -> Result<HttpResponse> {
    let uri = request.uri().to_string();
    let segments: Vec<&str> = uri
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    let size = query.size.unwrap_or(256);
    let resource_type = Resource::try_from(segments[0]);

    if let Ok(resource) = resource_type {
        let id = &path.0;
        let hash = &path.1;

        let key = format!("{resource}:{id}:{hash}:{size}");
        let cdn = data.get_ref();

        let mut is_from_cache = false;

        let image_data = if let Some(data) = cdn.cache.get(&key) {
            is_from_cache = true;

            Some(data)
        } else if let Some(data) = cdn.storage.get(resource, id, hash) {
            Some(data)
        } else {
            None
        };

        return match image_data {
            Some(image_data) => {
                let mut reader = Reader::new(Cursor::new(&image_data));

                reader.set_format(ImageFormat::WebP);

                let mut image = unwrap_or_return!(
                    reader.decode(),
                    ErrorInternalServerError("Decoding of image failed")
                );

                if !is_from_cache {
                    image = image.resize_exact(size, size, FilterType::Triangle);
                }

                let mut buffer = Cursor::new(Vec::new());

                unwrap_or_return!(
                    image.write_to(&mut buffer, ImageFormat::WebP),
                    ErrorInternalServerError("Buffer overflow")
                );

                let bytes = buffer.into_inner();

                if !is_from_cache {
                    unwrap_or_return!(
                        cdn.cache.put(key.to_owned(), &bytes),
                        ErrorInternalServerError("Failed to write to cache")
                    );
                }

                Ok(HttpResponse::Ok().content_type("image/webp").body(bytes))
            }
            None => Ok(
                HttpResponse::NotFound().body("Unsuccessful, the requested image was not found")
            ),
        };
    }

    Ok(HttpResponse::NotFound().finish())
}
