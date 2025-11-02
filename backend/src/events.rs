use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SystemEvent {
    #[serde(rename = "mqtt_message")]
    MqttMessage { topic: String, payload: String },
    
    #[serde(rename = "serial_status")]
    SerialStatus {
        connected: bool,
        port: Option<String>,
        baud_rate: Option<u32>,
        board_name: Option<String>,
    },
    
    #[serde(rename = "sensor_info")]
    SensorInfo {
        sensors: Vec<SensorDetail>,
        board: String,
        firmware: String,
    },
    
    #[serde(rename = "output")]
    Output { content: String },
    
    #[serde(rename = "error")]
    Error { source: String, message: String },
    
    #[serde(rename = "connected")]
    Connected,
    
    #[serde(rename = "mode_changed")]
    ModeChanged { mode: String },

    #[serde(rename = "transport_changed")]
    TransportChanged {
        transport: String,
        publish_topic: String,
        subscribe_topics: Vec<String>,
        // Optional board id derived from current serial board name (lowercased, spaces->underscore)
        #[serde(skip_serializing_if = "Option::is_none")]
        board_id: Option<String>,
    },

    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorDetail {
    pub id: u8,
    pub name: String,
    pub pin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientCommand {
    #[serde(rename = "command")]
    Command { command: String },
    
    #[serde(rename = "mode")]
    ChangeMode { mode: String },
    
    #[serde(rename = "subscribe")]
    Subscribe { topic: String },
    
    #[serde(rename = "publish")]
    Publish { topic: String, payload: String },
}
