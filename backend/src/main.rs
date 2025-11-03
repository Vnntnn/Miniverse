use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer};

mod config;
mod events;
mod mqtt;
mod serial;
mod state;
mod websocket;

use config::Config;
use events::{SystemEvent, SensorDetail};
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
    // Initialize default MQTT topics list in state
    state.get_ref().init_defaults().await;

    log::info!("Starting MQTT listener in separate thread...");
    let mqtt_state = state.clone();
    tokio::spawn(async move {
        log::info!("MQTT listener started");
        loop {
            match event_loop.poll().await {
                Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(p))) => {
                    let topic = p.topic.clone();
                    let payload = String::from_utf8_lossy(&p.payload).to_string();

                    // Parse info/state into structured event
                    if let Some(caps) = topic.strip_prefix("miniverse/") {
                        let parts: Vec<&str> = caps.split('/').collect();
                        if parts.len() >= 3 && parts[1] == "info" && parts[2] == "state" {
                            // Expected payload format:
                            //   SENSORS:TYPE:PIN,TYPE:PIN;BOARD:Name;FIRMWARE:Ver
                            let mut board = String::from("Unknown");
                            let mut firmware = String::from("Unknown");
                            let mut sensors: Vec<SensorDetail> = Vec::new();
                            for seg in payload.split(';') {
                                if let Some(s) = seg.strip_prefix("SENSORS:") {
                                    for (i, sp) in s.split(',').enumerate() {
                                        let kv: Vec<&str> = sp.split(':').collect();
                                        if kv.len() == 2 {
                                            sensors.push(SensorDetail {
                                                id: (i + 1) as u8,
                                                name: kv[0].trim().to_string(),
                                                pin: format!("Pin {}", kv[1].trim()),
                                            });
                                        }
                                    }
                                } else if let Some(b) = seg.strip_prefix("BOARD:") {
                                    board = b.trim().to_string();
                                } else if let Some(f) = seg.strip_prefix("FIRMWARE:") {
                                    firmware = f.trim().to_string();
                                }
                            }
                            mqtt_state.broadcast(SystemEvent::SensorInfo { sensors, board, firmware });
                            continue;
                        }
                    }

                    mqtt_state.broadcast(SystemEvent::MqttMessage { topic, payload });
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
            .route("/api/ports", web::get().to(api_ports))
            .route("/health", web::get().to(|| async { "OK" }))
            .service(Files::new("/", "../frontend/dist").index_file("index.html"))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}

async fn api_ports() -> HttpResponse {
    match SerialBridge::list_ports() {
        Ok(ports) => HttpResponse::Ok().json(ports),
        Err(e) => HttpResponse::InternalServerError().json(format!("Error: {}", e)),
    }
}
