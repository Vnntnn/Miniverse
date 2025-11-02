use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub broker_host: String,
    pub broker_port: u16,
    pub client_id: String,
    pub default_topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mqtt: MqttConfig {
                broker_host: "localhost".to_string(),
                broker_port: 1883,
                client_id: "miniverse-backend".to_string(),
                default_topics: vec!["miniverse/#".to_string()],
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                cors_origins: vec!["http://localhost:4321".to_string()],
            },
        }
    }
}

impl Config {
    #[allow(dead_code)]
    pub fn from_env() -> Self { Self::default() }
}
