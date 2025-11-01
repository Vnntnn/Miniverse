use actix_files::Files;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_cors::Cors;
use env_logger::Env;

mod handlers;
mod models;
mod serial;
mod cli;

use handlers::websocket::websocket_handler;
use handlers::api::get_available_ports;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    log::info!("========== Starting Miniverse Discovery Terminal Backend ==========");
    log::info!("======= Server will be available at: http://localhost:8080 ========");

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:4321")
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec!["content-type", "authorization"])
            .supports_credentials();

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .route("/ws", web::get().to(websocket_handler))
            .route("/api/ports", web::get().to(get_available_ports))
            .service(Files::new("/", "../frontend/dist").index_file("index.html"))
    })
    .workers(2)
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
