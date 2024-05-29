#[macro_use]
extern crate rs_cdn;

use anyhow::Result;
use colored::Colorize;
use rs_cdn::config;
use rs_cdn::{cdn::Cdn, rest};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use rs_cdn::cache::Cache;
use rs_cdn::colors::{GREEN, MAGENTA, RED};
use rs_cdn::storage::Storage;

#[tokio::main]
async fn main() -> Result<()> {
    let address: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    let config = match config::get_config() {
        Ok(config) => config,
        Err(err) => error!("{err}"),
    };

    let version = env!("CARGO_PKG_VERSION");
    let version = format!("v{version}");

    println!(
        r"
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
        if config.firewall.is_enabled() {
            "enabled".truecolor(GREEN.0, GREEN.1, GREEN.2)
        } else {
            "disabled".truecolor(RED.0, RED.1, RED.2)
        }
    );

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

    if config.firewall.is_enabled() {
        let trusted_sources = &config.firewall.trusted_sources;

        info!(
            "Trusted sources ({}): [{}]",
            trusted_sources.len(),
            trusted_sources
                .iter()
                .map(|src| src.to_string().bold().to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
    }

    let storage_path = config
        .storage_path
        .clone()
        .unwrap_or("./uploads".to_string());

    let storage = Storage::new(&storage_path);
    let cache = Cache::new();
    let cdn = Arc::new(Cdn::new(storage, cache, config).connect());

    HttpServer::new(move || {
        let cors = Cors::default().allow_any_origin();

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(cdn.clone()))
            .configure(rest::configure_routes)
    })
    .bind(address)
    .unwrap_or_else(|why| error!("Can't bind to {:?}: {}", address, why))
    .run()
    .await
    .expect("Failed to run HttpServer");

    Ok(())
}
