#[macro_use]
extern crate rs_cdn;

use colored::Colorize;
use rs_cdn::{cdn::Cdn, rest};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use actix_web::{web, App, HttpServer};
use rs_cdn::cache::Cache;
use rs_cdn::storage::Storage;
use rs_cdn::colors::{RED, GREEN, MAGENTA};

#[tokio::main]
async fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let version = format!("v{version}");

    println!(r"
               rs-cdn {version}
     {}     {}
    {}
     {}     Optional features:
                » firewall ({})
    ",
        r"/\_/\".truecolor(MAGENTA.0, MAGENTA.1, MAGENTA.2),
        "https://github.com/chris9740/cdn".underline(),
        "( o.o )".truecolor(MAGENTA.0, MAGENTA.1, MAGENTA.2),
        "> ^ <".truecolor(MAGENTA.0, MAGENTA.1, MAGENTA.2),

        if cfg!(feature = "firewall") {
            "enabled".truecolor(GREEN.0, GREEN.1, GREEN.2)
        } else {
            "disabled".truecolor(RED.0, RED.1, RED.2)
        }
    );

    let storage_path = env::var("CDN_STORAGE_PATH").unwrap_or(String::from("./cdn"));
    let storage = Storage::new(&storage_path);
    let cache = Cache::new();
    let cdn = Arc::new(Cdn::new(storage, cache).connect());

    let address: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    let debug_mode = if cfg!(debug_assertions) {
        "enabled"
    } else {
        "disabled"
    };

    info!(
        "Starting HttpServer on {} {}",
        address,
        format!("(debug mode {})", debug_mode).truecolor(140, 140, 140)
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cdn.clone()))
            .configure(rest::configure_routes)
    })
    .bind(address)
    .unwrap_or_else(|why| error!("Can't bind to {:?}: {why}", address))
    .run()
    .await
    .expect("Failed to run HttpServer");
}
