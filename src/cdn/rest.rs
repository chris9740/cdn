use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::dev::Payload;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web::{Error, FromRequest, Result};
use image::{imageops::FilterType, io::Reader};
use serde::Deserialize;
use std::env;
use std::future::{ready, Ready};
use std::io::{Cursor, Read};
use std::{fmt::Display, fs::File, sync::Arc};
use strum::{EnumIter, IntoEnumIterator};

use super::Cdn;

#[derive(Debug, EnumIter)]
pub enum Resource {
    Avatars,
    Icons,
}

impl Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Resource::Avatars => "avatars",
            Resource::Icons => "icons",
        })
    }
}

impl TryFrom<&str> for Resource {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        for resource in Resource::iter() {
            let resource_str = resource.to_string();

            if resource_str == value {
                return Ok(resource);
            }
        }

        Err("Unknown resource specified".to_string())
    }
}

fn configure_resource(resource: Resource, cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(&resource.to_string())
            .route("{resource_id}/{resource_hash}", web::get().to(get_resource))
            .route("{resource_id}", web::put().to(push_resource)),
    );
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    configure_resource(Resource::Avatars, cfg);
    configure_resource(Resource::Icons, cfg);
}

#[derive(MultipartForm)]
struct Upload {
    file: TempFile,
}

async fn push_resource(
    _: Authorized,
    form: MultipartForm<Upload>,
    data: web::Data<Arc<Cdn>>,
) -> HttpResponse {
    const FILE_SIZE_LIMIT: usize = 1024 * 1024 * 20; // 20 MB

    match form.file.size {
        0 => return HttpResponse::BadRequest().finish(),
        size if size > FILE_SIZE_LIMIT => {
            return HttpResponse::BadRequest().body("Image is too large")
        }
        _ => {}
    };

    let temp_path = form.file.file.path();
    let mut f = File::open(temp_path).expect("Tempfile should exist");
    let metadata = f.metadata().expect("Failed to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];

    f.read_exact(&mut buffer).expect("Buffer overflow");

    match data.storage.put(Resource::Avatars, "some_id", buffer) {
        Ok(()) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Debug, Deserialize)]
struct QueryParams {
    size: Option<u32>,
}

async fn get_resource(
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

        let key = format!("{resource}:{id}:{hash}");
        let cdn = data.get_ref();

        let image_data = if let Some(data) = cdn.cache.get(&key) {
            Some(data)
        } else if let Some(data) = cdn.storage.get(resource, id, hash) {
            cdn.cache
                .put(key.to_owned(), data.clone())
                .or(Err(ErrorInternalServerError("Cache failure")))?;

            Some(data)
        } else {
            None
        };

        return match image_data {
            Some(image_data) => {
                let reader = Reader::new(Cursor::new(&image_data)).with_guessed_format()?;

                let format = reader
                    .format()
                    .ok_or(ErrorBadRequest("Could not determine file format"))?;

                let image = reader
                    .decode()
                    .or(Err(ErrorInternalServerError("Decoding of image failed")))?;

                let image = image.resize_exact(size, size, FilterType::Triangle);
                let mut buffer = Cursor::new(Vec::new());

                image
                    .write_to(&mut buffer, format)
                    .or(Err(ErrorInternalServerError("Buffer overflow")))?;

                let bytes = buffer.into_inner();

                Ok(HttpResponse::Ok().content_type("image/png").body(bytes))
            }
            None => Ok(
                HttpResponse::NotFound().body("Unsuccessful, the requested image was not found")
            ),
        };
    }

    Ok(HttpResponse::NotFound().finish())
}

struct Authorized;

impl FromRequest for Authorized {
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if is_authorized(req) {
            ready(Ok(Authorized))
        } else {
            ready(Err(ErrorUnauthorized("Not authorized")))
        }
    }
}

fn is_authorized(req: &HttpRequest) -> bool {
    match req.headers().get("Authorization") {
        Some(value) => {
            if let Ok(secret) = env::var("CDN_SECRET") {
                let header_value = value.to_str().unwrap_or_default();

                return !header_value.is_empty() && header_value == secret;
            }

            false
        }
        None => false,
    }
}
