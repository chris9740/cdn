#[macro_use]
extern crate rs_cdn;

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use rs_cdn::{cdn::Cdn, rest};

use actix_web::{HttpServer, App, web};
use rs_cdn::cache::Cache;
use rs_cdn::storage::Storage;

#[tokio::main]
async fn main() {
    let storage_path = env::var("CDN_STORAGE_PATH").unwrap_or(String::from("./cdn"));
    let storage = Storage::new(&storage_path);
    let cache = Cache::new();
    let cdn = Arc::new(Cdn::new(storage, cache).connect());

    let address: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    let debug_mode = if cfg!(debug_assertions) { "enabled" } else { "disabled" };

    println!("Starting HttpServer on {} (debug mode {})", address, debug_mode);

    HttpServer::new(move || {
        App::new().app_data(web::Data::new(cdn.clone()))
            .configure(rest::configure_routes)
    })
    .bind(address)
    .unwrap_or_else(|why| error!("Can't bind to {:?}: {why}", address))
    .run()
    .await
    .expect("Failed to run HttpServer");
}
