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
) -> Result<HttpResponse> {
    const FILE_SIZE_LIMIT: usize = 1024 * 1024 * 20; // 20 MB

    let id = &path.as_str();

    match form.file.size {
        0 => return Ok(HttpResponse::BadRequest().finish()),
        size if size > FILE_SIZE_LIMIT => {
            return Ok(HttpResponse::BadRequest().body("Image is too large"))
        }
        _ => {}
    };

    let temp_path = form.file.file.path();
    let mut f = File::open(temp_path)?;
    let metadata = f.metadata()?;
    let mut buffer = vec![0; metadata.len() as usize];

    f.read_exact(&mut buffer)?;

    Ok(match data.storage.put(Resource::Avatars, id, buffer) {
        Ok(hash) => HttpResponse::Created().json(UploadResponse { hash }),
        Err(why) => HttpResponse::InternalServerError().body(why.to_string()),
    })
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
        Some(header) => {
            let secret = env::var("CDN_SECRET").unwrap_or("d3v_secret".to_string());

            header
                .to_str()
                .map(|header_value| header_value == secret)
                .unwrap_or(false)
        }
        None => false,
    }
}
