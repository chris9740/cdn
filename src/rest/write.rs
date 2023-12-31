use actix_multipart::{Multipart, MultipartError};
use actix_web::{web, HttpResponse};
#[cfg(feature = "firewall")]
use actix_web::HttpRequest;
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
use std::fs;
use std::str::Utf8Error;
use std::sync::Arc;
use thiserror::Error;

use crate::cdn::{Cdn, Connected};
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
    #[error("Internal server error")]
    InternalError,
    #[error("Unauthorized: {0}")]
    Unauthorized(&'static str)
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
            UploadError::Unauthorized(_) => HttpResponse::Unauthorized().json(GenericError {
                error: self.to_string()
            }),
            _ => HttpResponse::InternalServerError().finish()
        }
    }
}

const ONE_MB: usize = 1024 * 1024;
const FILE_SIZE_LIMIT: usize = ONE_MB * 20;

pub async fn push_resource(
    path: web::Path<String>,
    mut payload: Multipart,
    data: web::Data<Arc<Cdn<Connected>>>,
    #[cfg(feature = "firewall")]
    req: HttpRequest
) -> Result<HttpResponse, UploadError> {
    #[cfg(feature = "firewall")]
    {
        let whitelist = std::env::var("IP_WHITELIST").unwrap_or_default();
        let whitelist = whitelist.split(',').collect::<Vec<&str>>();

        let peer_addr = req.peer_addr().unwrap().ip();
        let source_addr = req.headers().get("X-Real-IP");

        let ip_addr = match source_addr {
            Some(source_addr) => {
                let peer_trusted = peer_addr.is_loopback();

                // This means the request is coming from nginx, or it's in a development
                // environment.
                // In either case, it's secure to trust the header.
                if peer_trusted {
                    source_addr
                        .to_str()
                        .map_err(|_| UploadError::InternalError)?
                        .parse()
                        .map_err(|_| UploadError::InternalError)?
                } else {
                    peer_addr
                }
            },
            None => {
                peer_addr
            }
        };

        if !whitelist.contains(&ip_addr.to_string().as_str()) {
            return Err(UploadError::Unauthorized("unknown remote address"));
        }
    }

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
        let limit = FILE_SIZE_LIMIT / ONE_MB;

        return Ok(HttpResponse::BadRequest().json(GenericError {
            error: format!("Image file size exceeds the limit of {limit}MB"),
        }));
    }

    let digest = openssl::sha::sha1(&image);
    let hash = hex::encode(digest);

    let decoded_signature = general_purpose::STANDARD
        .decode(signature)
        .map_err(|_| UploadError::Base64Error)?;

    if !verify_signature(&image, &decoded_signature)? {
        return Err(UploadError::Unauthorized("invalid signature"));
    }

    Ok(
        match data.storage.put(Resource::Avatars, id, image, &hash) {
            Ok(filename) => HttpResponse::Created().json(UploadResponse { filename }),
            Err(why) => HttpResponse::InternalServerError().json(why.to_string()),
        },
    )
}

fn verify_signature(data: &[u8], signature: &[u8]) -> Result<bool, ErrorStack> {
    let pkey_path = std::env::var("PUBLIC_KEY_PATH").unwrap_or("./certs/staging.pub".to_string());
    let pkey = fs::read_to_string(pkey_path).expect("Unable to load public key");
    let pkey = PKey::public_key_from_pem(pkey.as_bytes())?;
    let mut verifier = Verifier::new(MessageDigest::sha1(), &pkey)?;

    verifier.update(data)?;
    verifier.verify(signature)
}
