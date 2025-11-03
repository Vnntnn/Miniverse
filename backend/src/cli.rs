use crate::models::{SessionType, SystemInfo, WebSocketMessage};
use crate::serial::SerialManager;
use chrono::Utc;

pub struct CLIProcessor {
    session_type: SessionType,
    system_info: SystemInfo,
}

impl CLIProcessor {
    pub fn new() -> Self {
        Self {
            session_type: SessionType::Normal,
            system_info: SystemInfo::default(),
        }
    }

    pub fn process_command(&mut self, command: &str, serial_manager: &mut SerialManager) -> Vec<WebSocketMessage> {
        let mut responses = Vec::new();
        let trimmed = command.trim();

        if trimmed.is_empty() {
            return responses;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match self.session_type {
            SessionType::Normal => {
                responses.extend(self.process_normal_command(&cmd, &parts, serial_manager));
            },
            SessionType::Config => {
                responses.extend(self.process_config_command(&cmd, &parts, serial_manager));
            },
        }

        responses
    }

    fn process_normal_command(&mut self, cmd: &str, _parts: &[&str], _serial_manager: &mut SerialManager) -> Vec<WebSocketMessage> {
        let mut responses = Vec::new();

        match cmd {
            "help" => {
                let help_text = self.generate_help_text(SessionType::Normal);
                responses.push(WebSocketMessage::Output {
                    content: help_text,
                    timestamp: Utc::now(),
                    session_type: SessionType::Normal,
                });
            },
            "./info" | "info" => {
                let info_text = self.generate_system_info();
                responses.push(WebSocketMessage::Output {
                    content: info_text,
                    timestamp: Utc::now(),
                    session_type: SessionType::Normal,
                });
            },
            "config" => {
                self.session_type = SessionType::Config;
                responses.push(WebSocketMessage::ModeChanged { mode: SessionType::Config });
                responses.push(WebSocketMessage::Output {
                    content: "Entered configuration mode. Type 'help' for available commands.".to_string(),
                    timestamp: Utc::now(),
                    session_type: SessionType::Config,
                });
            },
            "clear" => {
                responses.push(WebSocketMessage::Output {
                    content: "".to_string(),
                    timestamp: Utc::now(),
                    session_type: SessionType::Normal,
                });
            },
            _ => {
                responses.push(WebSocketMessage::Output {
                    content: format!("Unknown command: '{}'. Type 'help' for available commands.", cmd),
                    timestamp: Utc::now(),
                    session_type: SessionType::Normal,
                });
            }
        }

        responses
    }

    fn process_config_command(&mut self, cmd: &str, parts: &[&str], serial_manager: &mut SerialManager) -> Vec<WebSocketMessage> {
        let mut responses = Vec::new();

        match cmd {
            "help" => {
                let help_text = self.generate_help_text(SessionType::Config);
                responses.push(WebSocketMessage::Output {
                    content: help_text,
                    timestamp: Utc::now(),
                    session_type: SessionType::Config,
                });
            },
            "scan" => {
                match SerialManager::list_ports() {
                    Ok(ports) => {
                        let port_text = if ports.is_empty() {
                            "No serial ports detected.".to_string()
                        } else {
                            let mut text = "Available serial ports:\n\n".to_string();
                            for (i, port) in ports.iter().enumerate() {
                                text.push_str(&format!("[{}] {}\n", i + 1, port.port_name));
                                
                                if let Some(desc) = &port.description {
                                    text.push_str(&format!("    Description: {}\n", desc));
                                }
                                if let Some(manufacturer) = &port.manufacturer {
                                    text.push_str(&format!("    Manufacturer: {}\n", manufacturer));
                                }
                                if let (Some(vid), Some(pid)) = (port.vendor_id, port.product_id) {
                                    text.push_str(&format!("    VID:PID = {:04X}:{:04X}\n", vid, pid));
                                }
                                text.push('\n');
                            }
                            text
                        };
                        responses.push(WebSocketMessage::Output {
                            content: port_text,
                            timestamp: Utc::now(),
                            session_type: SessionType::Config,
                        });
                        responses.push(WebSocketMessage::PortList { ports });
                    },
                    Err(e) => {
                        responses.push(WebSocketMessage::Error {
                            message: format!("Failed to scan ports: {}", e),
                        });
                    }
                }
            },
            "connect" => {
                if parts.len() < 2 {
                    responses.push(WebSocketMessage::Output {
                        content: "Usage: connect <port> [baud_rate]\nExample: connect /dev/ttyACM0 9600".to_string(),
                        timestamp: Utc::now(),
                        session_type: SessionType::Config,
                    });
                } else {
                    let port_name = parts[1];
                    let baud_rate = if parts.len() > 2 {
                        parts[2].parse().unwrap_or(9600)
                    } else {
                        9600
                    };

                    let result = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(
                            serial_manager.connect(port_name, baud_rate)
                        )
                    });

                    match result {
                        Ok(_) => {
                            responses.push(WebSocketMessage::Connected {
                                port: port_name.to_string(),
                            });
                            responses.push(WebSocketMessage::Output {
                                content: format!("Connected to {} at {} baud", port_name, baud_rate),
                                timestamp: Utc::now(),
                                session_type: SessionType::Config,
                            });
                        },
                        Err(e) => {
                            responses.push(WebSocketMessage::Error {
                                message: format!("Failed to connect: {}", e),
                            });
                        }
                    }
                }
            },
            "disconnect" => {
                if serial_manager.is_connected() {
                    let port_name = serial_manager.get_connected_port().unwrap_or("unknown").to_string();
                    serial_manager.disconnect();
                    responses.push(WebSocketMessage::Disconnected);
                    responses.push(WebSocketMessage::Output {
                        content: format!("Disconnected from {}", port_name),
                        timestamp: Utc::now(),
                        session_type: SessionType::Config,
                    });
                } else {
                    responses.push(WebSocketMessage::Output {
                        content: "Not connected to any port".to_string(),
                        timestamp: Utc::now(),
                        session_type: SessionType::Config,
                    });
                }
            },
            "status" => {
                let status_text = if let Some(port) = serial_manager.get_connected_port() {
                    format!("Connected to: {}", port)
                } else {
                    "Not connected".to_string()
                };
                responses.push(WebSocketMessage::Output {
                    content: status_text,
                    timestamp: Utc::now(),
                    session_type: SessionType::Config,
                });
            },
            "exit" => {
                self.session_type = SessionType::Normal;
                responses.push(WebSocketMessage::ModeChanged { mode: SessionType::Normal });
                responses.push(WebSocketMessage::Output {
                    content: "Exited configuration mode.".to_string(),
                    timestamp: Utc::now(),
                    session_type: SessionType::Normal,
                });
            },
            _ => {
                responses.push(WebSocketMessage::Output {
                    content: format!("Unknown command: '{}'. Type 'help' for available commands.", cmd),
                    timestamp: Utc::now(),
                    session_type: SessionType::Config,
                });
            }
        }

        responses
    }

    fn generate_help_text(&self, session: SessionType) -> String {
        match session {
            SessionType::Normal => {
                r#"
Available Commands:
------------------
help         Show this help message
./info       Display system information
config       Enter configuration mode
clear        Clear terminal screen

Type 'config' to manage serial connections.
"#.to_string()
            },
            SessionType::Config => {
                r#"
Configuration Mode Commands:
---------------------------
help                    Show this help message
scan                    Scan for available serial ports
connect <port> [baud]   Connect to a serial port
disconnect              Disconnect from current port
status                  Show connection status
exit                    Return to normal mode

Examples:
  scan                      Find available ports
  connect /dev/ttyACM0      Connect with default 9600 baud
  connect /dev/ttyACM0 115200  Connect with custom baud rate
"#.to_string()
            }
        }
    }

    fn generate_system_info(&self) -> String {
        format!(
            r#"
System Information
------------------
Project:  {}
Version:  {}
Platform: {}

Backend:     Rust + Actix Web
Frontend:    Astro + TypeScript
Connection:  WebSocket Real-time
Serial:      tokio-serial

Type 'help' for available commands.
Type 'config' to manage Arduino connections.
"#,
            self.system_info.project_name,
            self.system_info.version,
            if cfg!(target_os = "macos") { "macOS" } 
            else if cfg!(target_os = "linux") { "Linux" }
            else if cfg!(target_os = "windows") { "Windows" }
            else { "Unknown" }
        )
    }
}
