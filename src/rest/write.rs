use actix_multipart::{Multipart, MultipartError};
use actix_web::{web, HttpResponse};
use actix_web::{ResponseError, Result};
use base64::Engine;
use base64::engine::general_purpose;
use futures_util::StreamExt;
use image::EncodableLayout;
use openssl::error::ErrorStack;
use openssl::rsa::{Rsa, Padding};
use serde::{Serialize, Deserialize};
use std::str::Utf8Error;
use std::sync::Arc;
use thiserror::Error;

use crate::cdn::Cdn;
use crate::rest::Resource;

use super::{Authorized, GenericError};

#[derive(Serialize)]
pub struct UploadResponse {
    pub filename: String,
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum UploadError {
    #[error("Invalid public key")]
    InvalidPubKey(#[from] ErrorStack),
    #[error("Multipart error, {0}")]
    MultipartError(#[from] MultipartError),
    #[error("Could not decode utf8 data")]
    Utf8Error(#[from] Utf8Error),
    #[error("Metadata could not be parsed")]
    SerdeError(#[from] serde_json::Error),
    #[error("Missing field {0} in body")]
    MissingField(&'static str),
}

impl ResponseError for UploadError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match *self {
            UploadError::SerdeError(ref err) => HttpResponse::BadRequest().json(GenericError {
                error: format!("Serde error: {}", err),
            }),
            UploadError::MissingField(ref field) => HttpResponse::BadRequest().json(GenericError {
                error: format!("Missing {} field in body", field),
            }),
            UploadError::InvalidPubKey(_) => {
                HttpResponse::InternalServerError().body("Public key error")
            }
            UploadError::MultipartError(ref err) => {
                HttpResponse::InternalServerError().json(GenericError {
                    error: format!("Multipart error: {}", err),
                })
            }
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}

const FILE_SIZE_LIMIT: usize = 1024 * 1024 * 20; // 20 MB

pub async fn push_resource(
    _: Authorized,
    path: web::Path<String>,
    mut payload: Multipart,
    data: web::Data<Arc<Cdn>>,
) -> Result<HttpResponse, UploadError> {
    let id = &path.as_str();

    let mut buffer = Vec::new();
    let mut metadata_encrypted = String::new();

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_type = field.content_disposition();

        match field.name() {
            "image" => {
                if content_type.get_filename().is_none() {
                    return Ok(HttpResponse::BadRequest().body("Image is not a file"));
                }

                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    buffer.extend(data);
                }
            }
            "metadata" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;

                    metadata_encrypted.push_str(std::str::from_utf8(data.as_bytes())?);
                }
            }
            _ => {
                return Ok(HttpResponse::BadRequest().body("Invalid payload field"));
            }
        }
    }

    if buffer.is_empty() {
        return Err(UploadError::MissingField("buffer"));
    }

    if metadata_encrypted.is_empty() {
        return Err(UploadError::MissingField("metadata"));
    }

    if buffer.len() > FILE_SIZE_LIMIT {
        return Ok(HttpResponse::BadRequest().body("Image is too big"));
    }

    let metadata = read_metadata(&metadata_encrypted)?;

    let digest = md5::compute(&buffer);
    let hash = format!("{digest:x}");

    if hash != metadata.hash {
        return Ok(HttpResponse::Unauthorized().body("Invalid image checksum"));
    }

    Ok(
        match data.storage.put(Resource::Avatars, id, buffer, &hash) {
            Ok(filename) => HttpResponse::Created().json(UploadResponse { filename }),
            Err(why) => HttpResponse::InternalServerError().body(why.to_string()),
        },
    )
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub hash: String,
}

const PUBLIC_KEY: &[u8; 451] = include_bytes!("../../certs/staging.pub");

pub fn read_metadata(cipher: &str) -> Result<Metadata, UploadError> {
    let public_key = Rsa::public_key_from_pem(PUBLIC_KEY)?;
    let mut metadata_buf = vec![0; public_key.size() as usize];

    let metadata_cipher_raw = general_purpose::STANDARD
        .decode(cipher)
        .expect("Could not decode metadata base64");

    public_key
        .public_decrypt(metadata_cipher_raw.as_bytes(), &mut metadata_buf, Padding::PKCS1)?;

    let end_of_data = metadata_buf.iter().position(|&x| x == 0).unwrap_or(metadata_buf.len());

    metadata_buf.truncate(end_of_data);

    let metadata = std::str::from_utf8(&metadata_buf)?.trim();
    let metadata: Metadata = serde_json::from_str(metadata)?;

    Ok(metadata)
}
