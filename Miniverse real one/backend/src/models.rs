use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionType {
    Normal,
    Config,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "output")]
    Output {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        timestamp: Option<DateTime<Utc>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        session_type: Option<SessionType>,
    },

    #[serde(rename = "error")]
    Error { message: String },

    #[serde(rename = "mode_changed")]
    ModeChanged { mode: SessionType },

    #[serde(rename = "connected")]
    Connected { port: String },

    #[serde(rename = "disconnected")]
    Disconnected,

    #[serde(rename = "sensor_data")]
    SensorData {
        sensor: String,
        value: f32,
        unit: String,
        timestamp: DateTime<Utc>,
    },
}
