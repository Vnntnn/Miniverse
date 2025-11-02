use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

mod config;
mod events;
mod mqtt;
mod serial;
mod state;
mod websocket;

use config::Config;
use events::SystemEvent;
use mqtt::MqttManager;
use serial::SerialBridge;
use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    log::info!("=== Miniverse Backend Starting ===");

    log::info!("Initializing MQTT manager...");
    let (mqtt, mut event_loop) = MqttManager::new("localhost", 1883, "miniverse-backend");

    log::info!("Subscribing to MQTT topic: miniverse/#");
    if let Err(e) = mqtt.subscribe("miniverse/#").await {
        log::error!("MQTT subscribe failed: {}", e);
    }

    log::info!("Initializing serial bridge...");
    let serial = SerialBridge::new();

    log::info!("Creating application state...");
    let config = Config::default();
    let state = web::Data::new(AppState::new(config, mqtt, serial));

    log::info!("Starting MQTT listener in separate thread...");
    let mqtt_state = state.clone();
    tokio::spawn(async move {
        log::info!("MQTT listener started");
        loop {
            match event_loop.poll().await {
                Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(p))) => {
                    let topic = p.topic.clone();
                    let payload = String::from_utf8_lossy(&p.payload).to_string();

                    mqtt_state.broadcast(SystemEvent::MqttMessage {
                        topic,
                        payload,
                    });
                }
                Ok(_) => {}
                Err(e) => {
                    log::error!("MQTT error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    });

    let host = state.config.server.host.clone();
    let port = state.config.server.port;

    log::info!("Starting HTTP server on {}:{}...", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .route("/ws", web::get().to(websocket::ws_route))
            .route("/health", web::get().to(|| async { "OK" }))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
