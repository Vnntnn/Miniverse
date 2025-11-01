use actix_files::Files;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use env_logger::Env;
use std::path::Path;

mod handlers;
mod models;
mod serial;
mod cli;

use handlers::websocket::websocket_handler;
use handlers::api::get_available_ports;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    log::info!("Starting Miniverse Discovery Terminal Backend");
    log::info!("Server available at: http://localhost:8080");

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:4321")
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec!["content-type", "authorization"])
            .supports_credentials()
            .max_age(3600);

        let app = App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .route("/ws", web::get().to(websocket_handler))
            .route("/api/ports", web::get().to(get_available_ports));

        // Check if dist folder exists before trying to serve
        let dist_path = "../frontend/dist";
        if Path::new(dist_path).exists() && Path::new(dist_path).is_dir() {
            log::info!("Serving static files from {}", dist_path);
            app.service(Files::new("/", dist_path).index_file("index.html"))
        } else {
            log::info!("Development mode: Frontend on http://localhost:4321");
            app
        }
    })
    .workers(2)
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
