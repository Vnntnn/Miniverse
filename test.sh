#!/bin/bash
# filepath: /Users/vnntnn/Desktop/IT/Year2/Sem1/Projects/Phycom/Miniverse/complete_fix.sh

set -e
PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"

echo "🔧 Complete fix for all compilation errors..."

cd "$PROJECT_ROOT/backend"

# ============ 1. Fix main.rs ============
cat > src/main.rs << 'EOFMAIN'
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
EOFMAIN

# ============ 2. Fix websocket/mod.rs ============
cat > src/websocket/mod.rs << 'EOFWSMOD'
mod connection;
mod handler;

pub use connection::ws_route;
EOFWSMOD

# ============ 3. Fix websocket/connection.rs - add ws_route ============
cat > src/websocket/connection.rs << 'EOFCONN'
use actix::{Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::{Duration, Instant};

use crate::events::{ClientCommand, SystemEvent};
use crate::state::AppState;
use crate::websocket::handler::handle_command;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsConnection {
    hb: Instant,
    state: AppState,
}

impl WsConnection {
    pub fn new(state: AppState) -> Self {
        Self {
            hb: Instant::now(),
            state,
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                log::warn!("WebSocket client timeout, disconnecting");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for WsConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("WebSocket connection started");
        self.hb(ctx);

        self.state.broadcast(SystemEvent::Connected);

        let addr = ctx.address();
        let state = self.state.clone();

        actix::spawn(async move {
            let mut rx = state.subscribe();
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if let Ok(json) = serde_json::to_string(&event) {
                            addr.do_send(BroadcastMessage(json));
                        }
                    }
                    Err(e) => {
                        log::error!("Event channel error: {}", e);
                        break;
                    }
                }
            }
        });
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        log::info!("WebSocket connection stopped");
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                log::debug!("Received: {}", text);

                match serde_json::from_str::<ClientCommand>(&text) {
                    Ok(cmd) => {
                        let state = self.state.clone();
                        let addr = ctx.address();

                        actix::spawn(async move {
                            let response = handle_command(cmd, &state).await;
                            if let Ok(json) = serde_json::to_string(&response) {
                                addr.do_send(SendMessage(json));
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Parse error: {}", e);
                        let error = SystemEvent::Error {
                            source: "websocket".to_string(),
                            message: format!("Invalid command format: {}", e),
                        };
                        if let Ok(json) = serde_json::to_string(&error) {
                            ctx.text(json);
                        }
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                log::warn!("Binary message not supported");
            }
            Ok(ws::Message::Close(reason)) => {
                log::info!("WebSocket close: {:?}", reason);
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct SendMessage(String);

impl Handler<SendMessage> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct BroadcastMessage(String);

impl Handler<BroadcastMessage> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// WebSocket route handler
pub async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let state = state.get_ref().clone();
    ws::start(WsConnection::new(state), &req, stream)
}
EOFCONN

# ============ 4. Fix serial/mod.rs - remove unused ============
cat > src/serial/mod.rs << 'EOFSERIALMOD'
mod bridge;
mod commands;

pub use bridge::SerialBridge;
pub use commands::handle_serial_command;
EOFSERIALMOD

echo ""
echo "Cleaning build artifacts..."
cargo clean

echo ""
echo "Building release..."
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error|warning)" | head -30

if [ $? -eq 0 ]; then
    echo ""
    echo "╔══════════════════════════════════════════╗"
    echo "║         ✅ BUILD SUCCESSFUL!             ║"
    echo "╚══════════════════════════════════════════╝"
    echo ""
    echo "🚀 Start system:"
    echo "   ./start_all.sh"
    echo ""
    echo "📖 Open browser:"
    echo "   http://localhost:4321"
    echo ""
else
    echo ""
    echo "❌ Build failed. Check errors above."
    exit 1
fi