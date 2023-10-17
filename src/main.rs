use std::{env, net::SocketAddr, sync::Arc};

use actix_web::{web, App, HttpServer};
use cache::Cache;
use cdn::Cdn;
use storage::Storage;

mod cache;
mod cdn;
mod rest;
mod storage;

#[macro_use]
mod macros;

#[tokio::main]
async fn main() {
    let storage_path = env::var("CDN_STORAGE_PATH").unwrap_or(String::from("./cdn"));
    let storage = Storage::new(storage_path);
    let cache = Cache::new();
    let cdn = Arc::new(Cdn::new(storage, cache));

    let address: SocketAddr = "127.0.0.1:8080"
        .parse()
        .expect("Could not parse SocketAddr");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cdn.clone()))
            .configure(rest::configure_routes)
    })
    .bind(address)
    .unwrap_or_else(|why| error!("Can't bind to {:?}: {why}", address))
    .run();

    server.await.expect("Failed to start HttpServer");
}
