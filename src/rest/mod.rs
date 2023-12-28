pub mod read;
pub mod write;

use actix_web::{error::ErrorInternalServerError, web, HttpResponse, Result};
use serde::Serialize;
use std::{fmt::Display, sync::Arc};
use strum::{EnumIter, IntoEnumIterator};

use crate::{
    unwrap_or_return,
    rest::{read::get_resource, write::push_resource}, cdn::Connected,
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
            .route(
                r"{id}/{image_hash:(a_)?[0-9a-fA-F]{40}}.{ext:(png|gif)}",
                web::get().to(get_resource),
            )
            .route("{id}", web::post().to(push_resource)),
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

async fn get_health(data: web::Data<Arc<Cdn<Connected>>>) -> Result<HttpResponse> {
    let redis = data.redis();

    let mut con = unwrap_or_return!(
        redis.lock(),
        ErrorInternalServerError("Connection error with redis")
    );

    Ok(match data.cache.health(&mut con) {
        Ok(health) => HttpResponse::Ok().json(health),
        Err(why) => HttpResponse::InternalServerError().json(GenericError {
            error: why.to_string(),
        }),
    })
}
