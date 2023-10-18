pub mod read;
pub mod write;

use actix_web::{web, HttpResponse};
use serde::Serialize;
use std::{fmt::Display, sync::Arc};
use strum::{EnumIter, IntoEnumIterator};

use crate::rest::{read::get_resource, write::push_resource};

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

    cfg.route("health", web::get().to(get_health));
}

#[derive(Serialize)]
struct GenericError {
    error: String,
}

async fn get_health(data: web::Data<Arc<Cdn>>) -> HttpResponse {
    match data.cache.health() {
        Some(health) => HttpResponse::Ok().json(health),
        None => HttpResponse::InternalServerError().json(GenericError {
            error: String::from("Error reading from redis"),
        }),
    }
}
