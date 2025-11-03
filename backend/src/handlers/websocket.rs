use actix::{Actor, ActorContext, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "command")]
    Command { command: String },
    
    #[serde(rename = "mqtt_message")]
    MqttMessage { topic: String, payload: String },
    
    #[serde(rename = "output")]
    Output { content: String },
    
    #[serde(rename = "error")]
    Error { message: String },
    
    #[serde(rename = "connected")]
    Connected,
    
    #[serde(rename = "ping")]
    Ping,
    
    #[serde(rename = "pong")]
    Pong,
}

pub struct WebSocketConnection {
    hb: Instant,
    mqtt_rx: Option<mpsc::UnboundedReceiver<(String, String)>>,
}

impl WebSocketConnection {
    pub fn new(mqtt_rx: mpsc::UnboundedReceiver<(String, String)>) -> Self {
        Self {
            hb: Instant::now(),
            mqtt_rx: Some(mqtt_rx),
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

    fn start_mqtt_forwarding(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        if let Some(mut rx) = self.mqtt_rx.take() {
            let addr = ctx.address();
            actix::spawn(async move {
                while let Some((topic, payload)) = rx.recv().await {
                    addr.do_send(MqttForward { topic, payload });
                }
            });
        }
    }
}

impl Actor for WebSocketConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("WebSocket connection established");
        self.hb(ctx);
        self.start_mqtt_forwarding(ctx);

        let msg = WebSocketMessage::Connected;
        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        log::info!("WebSocket connection closed");
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketConnection {
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
                self.hb = Instant::now();
                
                match serde_json::from_str::<WebSocketMessage>(&text) {
                    Ok(WebSocketMessage::Command { command }) => {
                        ctx.notify(CommandMessage { command });
                    }
                    _ => {}
                }
            }
            Ok(ws::Message::Binary(_)) => {}
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct CommandMessage {
    command: String,
}

impl Handler<CommandMessage> for WebSocketConnection {
    type Result = ();

    fn handle(&mut self, msg: CommandMessage, _ctx: &mut Self::Context) {
        log::debug!("Received command: {}", msg.command);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct MqttForward {
    topic: String,
    payload: String,
}

impl Handler<MqttForward> for WebSocketConnection {
    type Result = ();

    fn handle(&mut self, msg: MqttForward, ctx: &mut Self::Context) {
        let ws_msg = WebSocketMessage::MqttMessage {
            topic: msg.topic,
            payload: msg.payload,
        };
        
        if let Ok(json) = serde_json::to_string(&ws_msg) {
            ctx.text(json);
        }
    }
}

pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<crate::AppState>,
) -> Result<HttpResponse, Error> {
    let (tx, rx) = mpsc::unbounded_channel();
    data.ws_clients.lock().unwrap().push(tx);
    
    ws::start(WebSocketConnection::new(rx), &req, stream)
}
