use std::{fmt::Display, sync::Arc};

use actix_web::{web, HttpRequest, HttpResponse};
use strum::{EnumIter, IntoEnumIterator};

use super::CDN;

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
            .route("{resource_id}/{resource_hash}", web::get().to(get_resource)),
    );
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    configure_resource(Resource::Avatars, cfg);
    configure_resource(Resource::Icons, cfg);
}

async fn get_resource(
    request: HttpRequest,
    path: web::Path<(String, String)>,
    data: web::Data<Arc<CDN>>,
) -> HttpResponse {
    let uri = request.uri().to_string();
    let segments: Vec<&str> = uri
        .split("/")
        .filter(|segment| !segment.is_empty())
        .collect();

    let resource_type = Resource::try_from(segments[0]);

    if let Ok(resource) = resource_type {
        let resource_id = &path.0;
        let resource_hash = &path.1;

        let key = format!("{}:{resource_id}:{resource_hash}", resource.to_string());
        let cdn = data.get_ref();

        let image_data = if let Some(data) = cdn.cache.get(&key) {
            Some(data)
        } else if let Some(data) = cdn.storage.get(resource, &resource_id, &resource_hash) {
            cdn.cache.put(key.to_owned(), data.clone());

            Some(data)
        } else {
            None
        };

        return match image_data {
            Some(image_data) => HttpResponse::Ok()
                .content_type("image/png")
                .body(image_data),
            None => HttpResponse::NotFound().finish(),
        };
    }

    HttpResponse::NotFound().finish()
}
