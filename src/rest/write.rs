use actix_multipart::{Multipart, MultipartError};
use actix_web::{web, HttpResponse};
use actix_web::{ResponseError, Result};
use base64::engine::general_purpose;
use base64::Engine;
use futures_util::StreamExt;
use image::EncodableLayout;
use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Verifier;
use serde::Serialize;
use std::str::Utf8Error;
use std::sync::Arc;
use thiserror::Error;

use crate::cdn::Cdn;
use crate::rest::Resource;

use super::GenericError;

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
    #[error("Signature could not be parsed")]
    SerdeError(#[from] serde_json::Error),
    #[error("Missing {0} field in body")]
    MissingField(&'static str),
    #[error("Base64 could not be decoded")]
    Base64Error,
}

impl ResponseError for UploadError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match *self {
            UploadError::SerdeError(_) => HttpResponse::BadRequest().json(GenericError {
                error: self.to_string(),
            }),
            UploadError::MissingField(_) => HttpResponse::BadRequest().json(GenericError {
                error: self.to_string(),
            }),
            UploadError::InvalidPubKey(_) => {
                HttpResponse::InternalServerError().body(self.to_string())
            }
            UploadError::MultipartError(_) => {
                HttpResponse::InternalServerError().json(GenericError {
                    error: self.to_string(),
                })
            }
            UploadError::Base64Error => HttpResponse::BadRequest().json(GenericError {
                error: self.to_string(),
            }),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}

const ONE_MB: usize = 1024 * 1024;
const FILE_SIZE_LIMIT: usize = ONE_MB * 20;

pub async fn push_resource(
    path: web::Path<String>,
    mut payload: Multipart,
    data: web::Data<Arc<Cdn>>,
) -> Result<HttpResponse, UploadError> {
    let id = &path.as_str();

    let mut image = Vec::new();
    let mut signature = String::new();

    let (image_field, signature_field) = ("image", "signature");

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_type = field.content_disposition();

        match field.name() {
            name if name == image_field => {
                if content_type.get_filename().is_none() {
                    return Ok(HttpResponse::BadRequest().json(GenericError {
                        error: "Image is not a file".to_string(),
                    }));
                }

                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    image.extend(data);
                }
            }
            name if name == signature_field => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;

                    signature.push_str(std::str::from_utf8(data.as_bytes())?);
                }
            }
            field_name => {
                return Ok(HttpResponse::BadRequest().json(GenericError {
                    error: format!("Invalid payload field \"{field_name}\""),
                }));
            }
        }
    }

    if image.is_empty() {
        return Err(UploadError::MissingField(image_field));
    }

    if signature.is_empty() {
        return Err(UploadError::MissingField(signature_field));
    }

    if image.len() > FILE_SIZE_LIMIT {
        return Ok(HttpResponse::BadRequest().json(GenericError {
            error: "Image is too big".to_string(),
        }));
    }

    let digest = md5::compute(&image);
    let hash = format!("{digest:x}");

    let decoded_signature = general_purpose::STANDARD
        .decode(signature)
        .map_err(|_| UploadError::Base64Error)?;

    if !verify_signature(&image, &decoded_signature)? {
        return Ok(HttpResponse::Unauthorized().json(GenericError {
            error: "Invalid signature".to_string(),
        }));
    }

    Ok(
        match data.storage.put(Resource::Avatars, id, image, &hash) {
            Ok(filename) => HttpResponse::Created().json(UploadResponse { filename }),
            Err(why) => HttpResponse::InternalServerError().json(why.to_string()),
        },
    )
}

#[cfg(debug_assertions)]
const PUBLIC_KEY: &[u8; 450] = include_bytes!("../../certs/staging.pub");

#[cfg(not(debug_assertions))]
const PUBLIC_KEY: &[u8; 450] = include_bytes!("../../certs/rs-cdn.pub");

fn verify_signature(data: &[u8], signature: &[u8]) -> Result<bool, ErrorStack> {
    let pkey = PKey::public_key_from_pem(PUBLIC_KEY)?;
    let mut verifier = Verifier::new(MessageDigest::md5(), &pkey)?;

    verifier.update(data)?;
    verifier.verify(&signature)
}
