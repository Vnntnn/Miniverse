use crate::config::Config;
use crate::events::SystemEvent;
use crate::mqtt::MqttManager;
use crate::serial::SerialBridge;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub mqtt: Arc<RwLock<MqttManager>>,
    pub serial: Arc<RwLock<SerialBridge>>,
    event_tx: broadcast::Sender<SystemEvent>,
}

impl AppState {
    pub fn new(config: Config, mqtt: MqttManager, serial: SerialBridge) -> Self {
        let (tx, _) = broadcast::channel(100);

        Self {
            config: Arc::new(config),
            mqtt: Arc::new(RwLock::new(mqtt)),
            serial: Arc::new(RwLock::new(serial)),
            event_tx: tx,
        }
    }

    pub fn broadcast(&self, event: SystemEvent) {
        let _ = self.event_tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.event_tx.subscribe()
    }
}
