use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::{Duration, Instant};

use crate::models::{WebSocketMessage, SessionType};
use crate::serial::SerialManager;
use crate::cli::CLIProcessor;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    ws::start(WebSocketSession::new(), &req, stream)
}

pub struct WebSocketSession {
    hb: Instant,
    serial_manager: SerialManager,
    cli_processor: CLIProcessor,
}

impl WebSocketSession {
    pub fn new() -> Self {
        Self {
            hb: Instant::now(),
            serial_manager: SerialManager::new(),
            cli_processor: CLIProcessor::new(),
        }
    }

    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                log::info!("WebSocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }

    fn send_welcome_message(&self, ctx: &mut ws::WebsocketContext<Self>) {
        let welcome = r#"
╔══════════════════════════════════════════════════════════════╗
║                       MINIVERSE DISCOVERY TERMINAL           ║
║                        Advanced Arduino Interface            ║
╚══════════════════════════════════════════════════════════════╝

Welcome to Physical Computing project, Please see Instructor down below.
Type 'help' for available commands.
Type './info' for system information.
Type 'config' to enter configuration mode.
"#;

        let msg = WebSocketMessage::Output {
            content: welcome.to_string(),
            timestamp: chrono::Utc::now(),
            session_type: SessionType::Normal,
        };

        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("WebSocket connection established");
        self.hb(ctx);
        self.send_welcome_message(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        log::info!("WebSocket connection closed");
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                self.hb = Instant::now();
                
                if let Ok(client_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                    match client_msg {
                        WebSocketMessage::Command { command } => {
                            // Process command synchronously - CLI processor is now sync
                            let responses = self.cli_processor.process_command(&command, &mut self.serial_manager);
                            
                            // Send all responses
                            for response in responses {
                                if let Ok(json) = serde_json::to_string(&response) {
                                    ctx.text(json);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            ws::Message::Binary(_) => {
                log::warn!("Unexpected binary message");
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => {}
        }
    }
}
