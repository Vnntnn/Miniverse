use crate::events::{ClientCommand, SystemEvent};
use crate::serial::handle_serial_command;
use crate::state::AppState;

pub async fn handle_command(cmd: ClientCommand, state: &AppState) -> SystemEvent {
    match cmd {
        ClientCommand::Command { command } => {
            handle_serial_command(&command, state).await
        }
        
        ClientCommand::ChangeMode { mode } => {
            state.broadcast(SystemEvent::ModeChanged { mode: mode.clone() });
            SystemEvent::Output {
                content: format!("Mode changed to: {}", mode),
            }
        }
        
        ClientCommand::Subscribe { topic } => {
            let mqtt = state.mqtt.read().await;
            match mqtt.subscribe(&topic).await {
                Ok(_) => SystemEvent::Output {
                    content: format!("Subscribed: {}", topic),
                },
                Err(e) => SystemEvent::Error {
                    source: "mqtt".to_string(),
                    message: e,
                },
            }
        }
        
        ClientCommand::Publish { topic, payload } => {
            let mqtt = state.mqtt.read().await;
            match mqtt.publish(&topic, payload.as_bytes()).await {
                Ok(_) => SystemEvent::Output {
                    content: format!("Published to {}", topic),
                },
                Err(e) => SystemEvent::Error {
                    source: "mqtt".to_string(),
                    message: e,
                },
            }
        }
    }
}
