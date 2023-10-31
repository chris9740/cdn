pub mod read;
pub mod write;

use actix_web::{
    dev::Payload,
    error::{ErrorInternalServerError, ErrorUnauthorized},
    web, Error, FromRequest, HttpRequest, HttpResponse, Result,
};
use futures_util::future::{ready, Ready};
use serde::Serialize;
use std::{env, fmt::Display, sync::Arc};
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    macros::{unwrap_or_return, error},
    rest::{read::get_resource, write::push_resource},
};

use super::Cdn;

#[derive(Debug, EnumIter)]
pub enum Resource {
    Avatars,
    Icons,
}

impl Resource {
    pub fn singleton(&self) -> bool {
        match self {
            Self::Avatars => true,
            Self::Icons => true,
        }
    }
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
            .route(r"{id}/{image_hash:(a_)?[0-9a-fA-F]{32}}.{ext:(png|gif)}", web::get().to(get_resource))
            .route("{id}", web::put().to(push_resource)),
    );
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    configure_resource(Resource::Avatars, cfg);
    configure_resource(Resource::Icons, cfg);

    cfg.route("health", web::get().to(get_health));
}

#[derive(Serialize)]
pub struct GenericError {
    pub error: String,
}

async fn get_health(_: Authorized, data: web::Data<Arc<Cdn>>) -> Result<HttpResponse> {
    let mut con = unwrap_or_return!(
        data.redis.get_connection(),
        ErrorInternalServerError("Redis connection error")
    );

    Ok(match data.cache.health(&mut con) {
        Ok(health) => HttpResponse::Ok().json(health),
        Err(why) => HttpResponse::InternalServerError().json(GenericError {
            error: why.to_string(),
        }),
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
            ready(Err(ErrorUnauthorized("Unauthorized.")))
        }
    }
}

fn is_authorized(req: &HttpRequest) -> bool {
    match req.headers().get("Authorization") {
        Some(header) => {
            let secret = env::var("CDN_SECRET").unwrap_or_else(|_| {
                if cfg!(debug_assertions) {
                    "d3v_secret".to_string()
                } else {
                    error!("The CDN_SECRET environment variable is missing from the current environment");
                }
            });

            header
                .to_str()
                .map(|header_value| header_value == secret)
                .unwrap_or(false)
        }
        None => false,
    }
}
