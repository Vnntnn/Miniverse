use crate::events::{SystemEvent, SensorDetail};
use crate::state::{AppState, Transport};
use std::borrow::Cow;
use crate::serial::SerialBridge;

pub async fn handle_serial_command(cmd: &str, state: &AppState) -> SystemEvent {
    handle_serial_command_with_transport(cmd, state, None).await
}

/// Session-aware handler: when `transport_override` is provided, it will be used
/// to route unknown device commands instead of the global AppState transport.
pub async fn handle_serial_command_with_transport(
    cmd: &str,
    state: &AppState,
    transport_override: Option<Transport>,
) -> SystemEvent {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let command = parts.first().copied().unwrap_or("");
    
    match command {
        // Config mode utilities
        "ports" => handle_ports().await,
        "connect" => handle_connect(&parts[1..], state).await,
        "disconnect" => handle_disconnect(state).await,
        "status" => handle_status(state).await,
        "transport" => handle_transport(&parts[1..], state).await,
        // MQTT utilities
        "mqtt" => handle_mqtt(&parts[1..], state).await,
        // Info and meta
        "info" => handle_info(state).await,
        "help" => SystemEvent::Output { content: backend_help_text() },
        "about" => SystemEvent::Output { content: "Miniverse Arduino Firmware - Physical Computing & IoT".to_string() },
        "version" => SystemEvent::Output { content: "Miniverse Firmware: v1.0.0".to_string() },
        // Device commands (Normal mode)
        "temp" => exec_temp(&parts[1..], state, transport_override).await,
        "distance" => exec_distance(&parts[1..], state, transport_override).await,
        "set" => exec_set(&parts[1..], state, transport_override).await,
        "light" => exec_light(&parts[1..], state, transport_override).await,
        "lcd" => exec_lcd(&parts[1..], state, transport_override).await,
        // Reject unknowns explicitly
        _ => SystemEvent::Error { source: "cli".to_string(), message: "Unknown command. Type 'help' for available commands.".to_string() },
    }
}

fn backend_help_text() -> String {
    let mut s = String::new();
    s.push_str("\n+------------------------------ HELP ------------------------------+\n");
    s.push_str("| System                 | help, clear, config, normal/exit       |\n");
    s.push_str("| Serial (config mode)   | ports, connect <n> [baud],             |\n");
    s.push_str("|                        | disconnect, status, /info              |\n");
    s.push_str("| Device (normal mode)   | temp <C|F|K>, distance [id]            |\n");
    s.push_str("| LED                    | light on/off, set light <0-255> [color]|\n");
    s.push_str("| LCD                    | lcd clear, lcd show \"a\" [\"b\"]     |\n");
    s.push_str("| Firmware Meta          | help, version, about, info             |\n");
    s.push_str("| Transport              | transport serial | transport mqtt      |\n");
    s.push_str("| MQTT                   | mqtt sub <topic>, mqtt unsub <topic>   |\n");
    s.push_str("+----------------------------------------------------------------+\n");
    s
}

async fn handle_transport(args: &[&str], state: &AppState) -> SystemEvent {
    let mode = args.get(0).copied().unwrap_or("");
    match mode.to_lowercase().as_str() {
        "serial" => {
            let mut t = state.transport.write().await;
            *t = Transport::Serial;
                // Notify UI
                state.broadcast(crate::events::SystemEvent::TransportChanged {
                    transport: "serial".to_string(),
                    publish_topic: "".to_string(),
                    subscribe_topics: vec![],
                });
            SystemEvent::Output { content: "Transport: serial".to_string() }
        }
        "mqtt" => {
            let mut t = state.transport.write().await;
            *t = Transport::Mqtt;
                // Compute topics from state (initialized with config defaults)
                let publish_topic = "miniverse/command".to_string();
                let subscribe_topics = state.mqtt_topics.read().await.clone();
                state.broadcast(crate::events::SystemEvent::TransportChanged {
                    transport: "mqtt".to_string(),
                    publish_topic: publish_topic.clone(),
                    subscribe_topics: subscribe_topics.clone(),
                });
            SystemEvent::Output { content: "Transport: mqtt".to_string() }
        }
        _ => SystemEvent::Output { content: "Usage: transport <serial|mqtt>".to_string() },
    }
}

async fn handle_mqtt(args: &[&str], state: &AppState) -> SystemEvent {
    let sub = args.get(0).copied().unwrap_or("");
    match sub {
        "sub" | "subscribe" => {
            let topic = match args.get(1) { Some(t) => *t, None => {
                return SystemEvent::Error { source: "mqtt".to_string(), message: "Usage: mqtt sub <topic>".to_string() };
            }};
            let mqtt = state.mqtt.read().await;
            match mqtt.subscribe(topic).await {
                Ok(_) => {
                    // Track topic in state and notify UI to refresh topics list
                    {
                        let mut topics = state.mqtt_topics.write().await;
                        if !topics.iter().any(|t| t == topic) {
                            topics.push(topic.to_string());
                        }
                    }
                    let current = state.mqtt_topics.read().await.clone();
                    state.broadcast(crate::events::SystemEvent::TransportChanged {
                        transport: "mqtt".to_string(),
                        publish_topic: "miniverse/command".to_string(),
                        subscribe_topics: current,
                    });
                    SystemEvent::Output { content: format!("MQTT: subscribed to {}", topic) }
                },
                Err(e) => SystemEvent::Error { source: "mqtt".to_string(), message: e },
            }
        }
        "unsub" | "unsubscribe" => {
            let topic = match args.get(1) { Some(t) => *t, None => {
                return SystemEvent::Error { source: "mqtt".to_string(), message: "Usage: mqtt unsub <topic>".to_string() };
            }};
            let mqtt = state.mqtt.read().await;
            match mqtt.unsubscribe(topic).await {
                Ok(_) => {
                    // Remove topic from state list if present
                    {
                        let mut topics = state.mqtt_topics.write().await;
                        if let Some(pos) = topics.iter().position(|t| t == topic) {
                            topics.remove(pos);
                        }
                    }
                    let current = state.mqtt_topics.read().await.clone();
                    state.broadcast(crate::events::SystemEvent::TransportChanged {
                        transport: "mqtt".to_string(),
                        publish_topic: "miniverse/command".to_string(),
                        subscribe_topics: current,
                    });
                    SystemEvent::Output { content: format!("MQTT: unsubscribed from {}", topic) }
                }
                Err(e) => SystemEvent::Error { source: "mqtt".to_string(), message: e },
            }
        }
        "pub" | "publish" => {
            let topic = match args.get(1) { Some(t) => *t, None => {
                return SystemEvent::Error { source: "mqtt".to_string(), message: "Usage: mqtt pub <topic> <payload>".to_string() };
            }};
            let payload = if args.len() > 2 { args[2..].join(" ") } else { String::new() };
            let mqtt = state.mqtt.read().await;
            match mqtt.publish(topic, payload.as_bytes()).await {
                Ok(_) => SystemEvent::Output { content: format!("MQTT: published to {}: {}", topic, payload) },
                Err(e) => SystemEvent::Error { source: "mqtt".to_string(), message: e },
            }
        }
        _ => SystemEvent::Output { content: "Usage:\n  mqtt sub <topic>\n  mqtt unsub <topic>\n  mqtt pub <topic> <payload>\n".to_string() },
    }
}

// ===== Device command executors =====

fn board_id_from_name(name: Option<&str>) -> String {
    let raw = name.unwrap_or("board1");
    let mut s = raw.to_lowercase().replace(' ', "_");
    if s.is_empty() { s = "board1".into(); }
    s
}

async fn publish_component_command(state: &AppState, component: &str, payload: &str) -> Result<(), String> {
    let serial = state.serial.read().await;
    let bid = board_id_from_name(serial.get_board_name());
    let topic = format!("miniverse/{}/{}/command", bid, component);
    let mqtt = state.mqtt.read().await;
    mqtt.publish(&topic, payload.as_bytes()).await
}

async fn exec_temp(args: &[&str], state: &AppState, transport_override: Option<Transport>) -> SystemEvent {
    // Validate optional unit
    let unit = args.get(0).map(|u| u.to_ascii_uppercase());
    if let Some(u) = &unit {
        if u != "C" && u != "F" && u != "K" {
            return SystemEvent::Error { source: "cli".into(), message: "Usage: temp <C|F|K>".into() };
        }
    }
    let payload = match unit { Some(u) => format!("temp {}", u), None => "temp".to_string() };
    match transport_override.unwrap_or(*state.transport.read().await) {
        Transport::Serial => forward_to_arduino(&payload, state).await,
        Transport::Mqtt => match publish_component_command(state, "temp", &payload).await {
            Ok(_) => SystemEvent::Output { content: format!("MQTT: {}", payload) },
            Err(e) => SystemEvent::Error { source: "mqtt".into(), message: e },
        },
    }
}

async fn exec_distance(args: &[&str], state: &AppState, transport_override: Option<Transport>) -> SystemEvent {
    let payload = if let Some(id) = args.get(0) { format!("distance {}", id) } else { "distance".to_string() };
    match transport_override.unwrap_or(*state.transport.read().await) {
        Transport::Serial => forward_to_arduino(&payload, state).await,
        Transport::Mqtt => match publish_component_command(state, "distance", &payload).await {
            Ok(_) => SystemEvent::Output { content: format!("MQTT: {}", payload) },
            Err(e) => SystemEvent::Error { source: "mqtt".into(), message: e },
        },
    }
}

async fn exec_set(args: &[&str], state: &AppState, transport_override: Option<Transport>) -> SystemEvent {
    // support: set light <0-255> [color]
    if args.get(0) != Some(&"light") {
        return SystemEvent::Error { source: "cli".into(), message: "Usage: set light <0-255> [color]".into() };
    }
    let val = match args.get(1).and_then(|v| v.parse::<u16>().ok()) { Some(v) if v <= 255 => v, _ => {
        return SystemEvent::Error { source: "cli".into(), message: "Usage: set light <0-255> [color]".into() };
    }};
    let color = args.get(2).copied();
    let payload = match color { Some(c) => format!("set light {} {}", val, c), None => format!("set light {}", val) };
    match transport_override.unwrap_or(*state.transport.read().await) {
        Transport::Serial => forward_to_arduino(&payload, state).await,
        Transport::Mqtt => match publish_component_command(state, "led", &payload).await {
            Ok(_) => SystemEvent::Output { content: format!("MQTT: {}", payload) },
            Err(e) => SystemEvent::Error { source: "mqtt".into(), message: e },
        },
    }
}

async fn exec_light(args: &[&str], state: &AppState, transport_override: Option<Transport>) -> SystemEvent {
    let sub = args.get(0).copied().unwrap_or("");
    match sub {
        "on" => exec_set(&["light", "255"], state, transport_override).await,
        "off" => exec_set(&["light", "0"], state, transport_override).await,
        _ => SystemEvent::Error { source: "cli".into(), message: "Usage: light <on|off>".into() },
    }
}

async fn exec_lcd(args: &[&str], state: &AppState, transport_override: Option<Transport>) -> SystemEvent {
    let sub = args.get(0).copied().unwrap_or("");
    match sub {
        "clear" => {
            let payload = "lcd clear".to_string();
            match transport_override.unwrap_or(*state.transport.read().await) {
                Transport::Serial => forward_to_arduino(&payload, state).await,
                Transport::Mqtt => match publish_component_command(state, "lcd", &payload).await {
                    Ok(_) => SystemEvent::Output { content: "LCD: cleared".into() },
                    Err(e) => SystemEvent::Error { source: "mqtt".into(), message: e },
                },
            }
        }
        "show" => {
            // take the rest of the line after 'lcd '
            let payload = format!("lcd {}", args.join(" "));
            match transport_override.unwrap_or(*state.transport.read().await) {
                Transport::Serial => forward_to_arduino(&payload, state).await,
                Transport::Mqtt => match publish_component_command(state, "lcd", &payload).await {
                    Ok(_) => SystemEvent::Output { content: "LCD: show".into() },
                    Err(e) => SystemEvent::Error { source: "mqtt".into(), message: e },
                },
            }
        }
        _ => SystemEvent::Error { source: "cli".into(), message: "Usage: lcd <clear|show \"a\" [\"b\"]>".into() },
    }
}

async fn handle_ports() -> SystemEvent {
    match SerialBridge::list_ports() {
        Ok(ports) if !ports.is_empty() => {
            let mut output = String::new();
            output.push_str("\nSerial Ports:\n");
            
            for p in ports {
                // Single line format: [index] port (device)
                output.push_str(&format!("[{}] {} ({})\n", p.index, p.port_name, p.board_name));
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
    
    // Forward to firmware: try 'INFO' first
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
        Err(_) => {
            // Try again using alias '/INFO' and longer timeout
            let _ = serial.send_command("/INFO");
            match serial.read_line(3000) {
                Ok(resp) if resp.starts_with("SENSORS:") => {
                    let sensor_data = resp.strip_prefix("SENSORS:").unwrap_or("");
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
                            } else { None }
                        })
                        .collect();
                    SystemEvent::SensorInfo {
                        sensors,
                        board: serial.get_board_name().unwrap_or("Unknown").to_string(),
                        firmware: "v1.0.0".to_string(),
                    }
                }
                _ => SystemEvent::Output { content: "No sensor info returned from board (timeout). If sensors aren't connected, that's ok. Use 'status' or try commands like 'light on' or 'temp'.".to_string() },
            }
        },
    }
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

async fn forward_via_transport(cmd: &str, state: &AppState) -> SystemEvent {
    let t = *state.transport.read().await;
    match t {
        Transport::Serial => forward_to_arduino(cmd, state).await,
        Transport::Mqtt => forward_to_mqtt(cmd, state).await,
    }
}

async fn forward_via_transport_override(
    cmd: &str,
    state: &AppState,
    transport_override: Option<Transport>,
) -> SystemEvent {
    if let Some(t) = transport_override {
        return match t {
            Transport::Serial => forward_to_arduino(cmd, state).await,
            Transport::Mqtt => forward_to_mqtt(cmd, state).await,
        };
    }
    forward_via_transport(cmd, state).await
}

async fn forward_to_mqtt(cmd: &str, state: &AppState) -> SystemEvent {
    let mqtt = state.mqtt.read().await;
    // Publish the raw command; device should act and (optionally) publish telemetry
    match mqtt.publish("miniverse/command", cmd.as_bytes()).await {
        Ok(_) => SystemEvent::Output { content: format!("MQTT: published '{}'", cmd) },
        Err(e) => SystemEvent::Error { source: "mqtt".to_string(), message: e },
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
