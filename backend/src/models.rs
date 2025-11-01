use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialPort {
    pub port_name: String,
    pub port_type: String,
    pub description: Option<String>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    // Client to Server
    Command { command: String },
    
    // Server to Client
    Output { 
        content: String, 
        timestamp: DateTime<Utc>,
        session_type: SessionType 
    },
    Error { message: String },
    Connected { port: String },
    Disconnected,
    ModeChanged { mode: SessionType },
    PortList { ports: Vec<SerialPort> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    Normal,
    Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub project_name: String,
    pub version: String,
    pub description: String,
    pub commands: Vec<CommandInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub session: SessionType,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            project_name: "Miniverse Discovery Terminal".to_string(),
            version: "2.0.0".to_string(),
            description: "Advanced Arduino Serial Communication Interface".to_string(),
            commands: vec![
                CommandInfo {
                    name: "help".to_string(),
                    description: "Show available commands".to_string(),
                    usage: "help [command]".to_string(),
                    session: SessionType::Normal,
                },
                CommandInfo {
                    name: "./info".to_string(),
                    description: "Display system information".to_string(),
                    usage: "./info".to_string(),
                    session: SessionType::Normal,
                },
                CommandInfo {
                    name: "config".to_string(),
                    description: "Enter configuration mode".to_string(),
                    usage: "config".to_string(),
                    session: SessionType::Normal,
                },
                CommandInfo {
                    name: "scan".to_string(),
                    description: "Scan for available serial ports".to_string(),
                    usage: "scan".to_string(),
                    session: SessionType::Config,
                },
                CommandInfo {
                    name: "connect".to_string(),
                    description: "Connect to a serial port".to_string(),
                    usage: "connect <port> [baud_rate]".to_string(),
                    session: SessionType::Config,
                },
                CommandInfo {
                    name: "exit".to_string(),
                    description: "Exit configuration mode".to_string(),
                    usage: "exit".to_string(),
                    session: SessionType::Config,
                },
            ],
        }
    }
}
