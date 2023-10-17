use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::dev::Payload;
use actix_web::error::ErrorUnauthorized;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web::{Error, FromRequest, Result};
use serde::Serialize;
use std::env;
use std::future::{ready, Ready};
use std::io::Read;
use std::{fs::File, sync::Arc};

use crate::cdn::Cdn;
use crate::rest::Resource;

#[derive(MultipartForm)]
pub struct UploadForm {
    pub file: TempFile,
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub hash: String,
}

pub async fn push_resource(
    _: Authorized,
    path: web::Path<String>,
    form: MultipartForm<UploadForm>,
    data: web::Data<Arc<Cdn>>,
) -> HttpResponse {
    const FILE_SIZE_LIMIT: usize = 1024 * 1024 * 20; // 20 MB

    let id = &path.as_str();

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

    match data.storage.put(Resource::Avatars, id, buffer) {
        Ok(hash) => HttpResponse::Created().json(UploadResponse { hash }),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub struct Authorized;

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
