use crate::config::Config;
use crate::events::SystemEvent;
use crate::mqtt::MqttManager;
use crate::serial::SerialBridge;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transport {
    Serial,
    Mqtt,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub mqtt: Arc<RwLock<MqttManager>>,
    pub serial: Arc<RwLock<SerialBridge>>,
    pub transport: Arc<RwLock<Transport>>, // preferred transport for device commands
    event_tx: broadcast::Sender<SystemEvent>,
}

impl AppState {
    pub fn new(config: Config, mqtt: MqttManager, serial: SerialBridge) -> Self {
        let (tx, _) = broadcast::channel(100);

        Self {
            config: Arc::new(config),
            mqtt: Arc::new(RwLock::new(mqtt)),
            serial: Arc::new(RwLock::new(serial)),
            transport: Arc::new(RwLock::new(Transport::Serial)),
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
