use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;

pub struct MqttManager {
    pub client: AsyncClient,
}

impl MqttManager {
    pub fn new(broker_host: &str, broker_port: u16, client_id: &str) -> (Self, rumqttc::EventLoop) {
        let mut options = MqttOptions::new(client_id, broker_host, broker_port);
        options.set_keep_alive(Duration::from_secs(30));

        let (client, event_loop) = AsyncClient::new(options, 100);

        (Self { client }, event_loop)
    }

    pub async fn subscribe(&self, topic: &str) -> Result<(), String> {
        self.client
            .subscribe(topic, QoS::AtLeastOnce)
            .await
            .map_err(|e| format!("Subscribe failed: {}", e))
    }

    pub async fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), String> {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload)
            .await
            .map_err(|e| format!("Publish failed: {}", e))
    }

    pub async fn unsubscribe(&self, topic: &str) -> Result<(), String> {
        self.client
            .unsubscribe(topic)
            .await
            .map_err(|e| format!("Unsubscribe failed: {}", e))
    }
}
