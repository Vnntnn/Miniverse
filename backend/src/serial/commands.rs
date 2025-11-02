use crate::events::{SystemEvent, SensorDetail};
use crate::state::AppState;
use crate::serial::SerialBridge;

pub async fn handle_serial_command(cmd: &str, state: &AppState) -> SystemEvent {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let command = parts.first().copied().unwrap_or("");
    
    match command {
        "ports" => handle_ports().await,
        "connect" => handle_connect(&parts[1..], state).await,
        "disconnect" => handle_disconnect(state).await,
        "status" => handle_status(state).await,
        // Friendly aliases and firmware meta
        "/help" => SystemEvent::Output { content: backend_help_text() },
        "/version" => SystemEvent::Output { content: "Miniverse Firmware: v1.0.0".to_string() },
        "/about" => SystemEvent::Output { content: "Miniverse Arduino Firmware - Physical Computing & IoT".to_string() },
        "/info" | "info" => handle_info(state).await,
        // Friendly user commands mapped to firmware
        "temp" => alias_temp(state).await,
    "light" => alias_light(&parts[1..], state).await,
        // known helpers kept for compatibility
    "read" => handle_read(&parts[1..], state).await,
    "led" => handle_led(&parts[1..], state).await,
        // otherwise, forward any command directly to Arduino
        _ => forward_to_arduino(cmd, state).await,
    }
}

fn backend_help_text() -> String {
    let mut s = String::new();
    s.push_str("\n+--------------------------- HELP ---------------------------+\n");
    s.push_str("| System                 | help, clear, config, normal/exit  |\n");
    s.push_str("| Serial (config mode)   | ports, connect <n> [baud],        |\n");
    s.push_str("|                        | disconnect, status, /info         |\n");
    s.push_str("| Sensors (normal mode)  | temp                              |\n");
    s.push_str("| LED                    | light on/off/toggle               |\n");
    s.push_str("+-----------------------------------------------------------+\n");
    s
}

async fn handle_ports() -> SystemEvent {
    match SerialBridge::list_ports() {
        Ok(ports) if !ports.is_empty() => {
            let mut output = String::new();
            output.push_str("\nSerial Ports:\n");
            
            for p in ports {
                // Single line format: [index] port (device)
                output.push_str(&format!(" [{}] {} ({})\n", p.index, p.port_name, p.board_name));
            }
            
            output.push_str("\nconnect <index> [baud]\n");
            
            SystemEvent::Output { content: output }
        }
        Ok(_) => SystemEvent::Output {
            content: "No serial ports found.".to_string(),
        },
        Err(e) => SystemEvent::Error {
            source: "serial".to_string(),
            message: e,
        },
    }
}

async fn handle_connect(args: &[&str], state: &AppState) -> SystemEvent {
    let index = args.get(0).and_then(|s| s.parse::<usize>().ok());
    let baud = args.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(115200);
    
    if let Some(idx) = index {
        match SerialBridge::list_ports() {
            Ok(ports) if idx < ports.len() => {
                let port_info = &ports[idx];
                
                let result = {
                    let mut serial = state.serial.write().await;
                    serial.connect(&port_info.port_name, baud, port_info.board_name.clone())
                };
                
                match result {
                    Ok(_) => {
                        state.broadcast(SystemEvent::SerialStatus {
                            connected: true,
                            port: Some(port_info.port_name.clone()),
                            baud_rate: Some(baud),
                            board_name: Some(port_info.board_name.clone()),
                        });
                        SystemEvent::Output {
                            content: format!(
                                "Connected: {} - {} @ {} baud",
                                port_info.port_name, port_info.board_name, baud
                            ),
                        }
                    }
                    Err(e) => SystemEvent::Error {
                        source: "serial".to_string(),
                        message: e,
                    },
                }
            }
            Ok(_) => SystemEvent::Error {
                source: "serial".to_string(),
                message: "Invalid port index".to_string(),
            },
            Err(e) => SystemEvent::Error {
                source: "serial".to_string(),
                message: e,
            },
        }
    } else {
        SystemEvent::Error {
            source: "serial".to_string(),
            message: "Usage: connect <index> [baud]".to_string(),
        }
    }
}

async fn handle_disconnect(state: &AppState) -> SystemEvent {
    let mut serial = state.serial.write().await;
    serial.disconnect();
    
    state.broadcast(SystemEvent::SerialStatus {
        connected: false,
        port: None,
        baud_rate: None,
        board_name: None,
    });
    
    SystemEvent::Output {
        content: "Disconnected.".to_string(),
    }
}

async fn handle_status(state: &AppState) -> SystemEvent {
    let serial = state.serial.read().await;
    let msg = if serial.is_connected() {
        format!(
            "Serial: {} - {} @ {} baud",
            serial.get_port_name().unwrap_or("?"),
            serial.get_board_name().unwrap_or("?"),
            serial.get_baud_rate()
        )
    } else {
        "Serial: Not connected".to_string()
    };
    SystemEvent::Output { content: msg }
}

async fn handle_info(state: &AppState) -> SystemEvent {
    let serial = state.serial.read().await;
    
    if !serial.is_connected() {
        return SystemEvent::Error {
            source: "serial".to_string(),
            message: "Not connected to board".to_string(),
        };
    }
    
    // Forward to firmware: use 'INFO'
    if let Err(e) = serial.send_command("INFO") {
        return SystemEvent::Error {
            source: "serial".to_string(),
            message: format!("Failed to send INFO: {}", e),
        };
    }
    
    // Read response (format: SENSORS:DHT22:2,LED:13,PIR:7)
    match serial.read_line(2000) {
        Ok(response) if response.starts_with("SENSORS:") => {
            let sensor_data = response.strip_prefix("SENSORS:").unwrap_or("");
            let sensors: Vec<SensorDetail> = sensor_data
                .split(',')
                .enumerate()
                .filter_map(|(i, s)| {
                    let parts: Vec<&str> = s.split(':').collect();
                    if parts.len() == 2 {
                        Some(SensorDetail {
                            id: (i + 1) as u8,
                            name: parts[0].to_string(),
                            pin: format!("Pin {}", parts[1]),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            
            SystemEvent::SensorInfo {
                sensors,
                board: serial.get_board_name().unwrap_or("Unknown").to_string(),
                firmware: "v1.0.0".to_string(),
            }
        }
        Ok(response) => SystemEvent::Error {
            source: "serial".to_string(),
            message: format!("Invalid response: {}", response),
        },
        Err(e) => SystemEvent::Error {
            source: "serial".to_string(),
            message: format!("Read failed: {}", e),
        },
    }
}

async fn alias_temp(state: &AppState) -> SystemEvent {
    let serial = state.serial.read().await;
    if !serial.is_connected() {
        return SystemEvent::Error { source: "serial".to_string(), message: "Not connected".to_string() };
    }
    if let Err(e) = serial.send_command("READ_TEMP") {
        return SystemEvent::Error { source: "serial".to_string(), message: format!("Send failed: {}", e) };
    }
    match serial.read_line(3000) {
        Ok(resp) => SystemEvent::Output { content: resp },
        Err(e) => SystemEvent::Error { source: "serial".to_string(), message: e },
    }
}

async fn alias_light(args: &[&str], state: &AppState) -> SystemEvent {
    let action = args.get(0).copied().unwrap_or("");
    let cmd = match action {
        "on" => Some("LED_ON"),
        "off" => Some("LED_OFF"),
        "toggle" => Some("LED_TOGGLE"),
        _ => None,
    };
    if let Some(cmd) = cmd {
        let serial = state.serial.read().await;
        if !serial.is_connected() {
            return SystemEvent::Error { source: "serial".to_string(), message: "Not connected".to_string() };
        }
        if let Err(e) = serial.send_command(cmd) {
            return SystemEvent::Error { source: "serial".to_string(), message: format!("Send failed: {}", e) };
        }
        return SystemEvent::Output { content: format!("Light {}", action) };
    }
    SystemEvent::Error { source: "serial".to_string(), message: "Usage: light <on|off|toggle>".to_string() }
}

async fn forward_to_arduino(cmd: &str, state: &AppState) -> SystemEvent {
    let serial = state.serial.read().await;
    if !serial.is_connected() {
        return SystemEvent::Error {
            source: "serial".to_string(),
            message: "Not connected".to_string(),
        };
    }

    if let Err(e) = serial.send_command(cmd) {
        return SystemEvent::Error {
            source: "serial".to_string(),
            message: format!("Send failed: {}", e),
        };
    }

    match serial.read_line(5000) {
        Ok(response) => SystemEvent::Output { content: response },
        Err(e) => SystemEvent::Error { source: "serial".to_string(), message: e },
    }
}

async fn handle_read(args: &[&str], state: &AppState) -> SystemEvent {
    let sensor = args.get(0).copied().unwrap_or("all");
    let serial = state.serial.read().await;
    
    if !serial.is_connected() {
        return SystemEvent::Error {
            source: "serial".to_string(),
            message: "Not connected".to_string(),
        };
    }
    
    let cmd = match sensor {
        "temp" | "temperature" => "READ_TEMP",
        "humidity" | "hum" => "READ_HUM",
        "all" => "READ_ALL",
        _ => return SystemEvent::Error {
            source: "serial".to_string(),
            message: format!("Unknown sensor: {}", sensor),
        },
    };
    
    if let Err(e) = serial.send_command(cmd) {
        return SystemEvent::Error {
            source: "serial".to_string(),
            message: e,
        };
    }
    
    match serial.read_line(2000) {
        Ok(response) => SystemEvent::Output { content: response },
        Err(e) => SystemEvent::Error {
            source: "serial".to_string(),
            message: e,
        },
    }
}

async fn handle_led(args: &[&str], state: &AppState) -> SystemEvent {
    let action = args.get(0).copied().unwrap_or("");
    let serial = state.serial.read().await;
    
    if !serial.is_connected() {
        return SystemEvent::Error {
            source: "serial".to_string(),
            message: "Not connected".to_string(),
        };
    }
    
    let cmd = match action {
        "on" => "LED_ON",
        "off" => "LED_OFF",
        "toggle" => "LED_TOGGLE",
        _ => return SystemEvent::Error {
            source: "serial".to_string(),
            message: "Usage: led <on|off|toggle>".to_string(),
        },
    };
    
    if let Err(e) = serial.send_command(cmd) {
        return SystemEvent::Error {
            source: "serial".to_string(),
            message: e,
        };
    }
    
    SystemEvent::Output {
        content: format!("LED {}", action),
    }
}
