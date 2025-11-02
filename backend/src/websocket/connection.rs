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

pub async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let state = state.get_ref().clone();
    ws::start(WsConnection::new(state), &req, stream)
}
