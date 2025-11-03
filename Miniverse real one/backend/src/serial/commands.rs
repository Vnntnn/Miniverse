use crate::events::{SystemEvent, SensorDetail};
use crate::state::{AppState, Transport};
use crate::serial::SerialBridge;

#[allow(dead_code)]
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
    // accept both 'light' and 'led' as aliases
    "light" | "led" => exec_light(&parts[1..], state, transport_override).await,
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
    s.push_str("|                        | disconnect, status                     |\n");
    s.push_str("| Device (normal mode)   | temp, distance [id]                   |\n");
    s.push_str("| LED                    | light on/off, set light <0-255> [color]|\n");
    s.push_str("| LCD                    | lcd clear, lcd show \"a\" [\"b\"]     |\n");
    s.push_str("| Firmware Meta          | help, version, about, info             |\n");
    s.push_str("| Transport              | transport serial | transport mqtt      |\n");
    s.push_str("| MQTT                   | mqtt sub <topic>, mqtt unsub <topic>,  |\n");
    s.push_str("|                        | mqtt subs (list)                        |\n");
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
                    board_id: {
                        let serial = state.serial.read().await;
                        Some(board_id_from_name(serial.get_board_name()))
                    },
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
                    board_id: {
                        let serial = state.serial.read().await;
                        Some(board_id_from_name(serial.get_board_name()))
                    },
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
                        board_id: {
                            let serial = state.serial.read().await;
                            Some(board_id_from_name(serial.get_board_name()))
                        },
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
                        board_id: {
                            let serial = state.serial.read().await;
                            Some(board_id_from_name(serial.get_board_name()))
                        },
                    });
                    SystemEvent::Output { content: format!("MQTT: unsubscribed from {}", topic) }
                }
                Err(e) => SystemEvent::Error { source: "mqtt".to_string(), message: e },
            }
        }
        "subs" => {
            // List current subscriptions from state
            let current = state.mqtt_topics.read().await.clone();
            if current.is_empty() {
                SystemEvent::Output { content: "MQTT: no subscriptions".to_string() }
            } else {
                SystemEvent::Output { content: format!("MQTT Subscriptions ({}):\n{}", current.len(), current.join("\n")) }
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
        _ => SystemEvent::Output { content: "Usage:\n  mqtt sub <topic>\n  mqtt unsub <topic>\n  mqtt subs\n  mqtt pub <topic> <payload>\n".to_string() },
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

fn component_topic_sync(serial: &crate::serial::SerialBridge, component: &str) -> String {
    let bid = board_id_from_name(serial.get_board_name());
    format!("miniverse/{}/{}/command", bid, component)
}

async fn exec_temp(_args: &[&str], state: &AppState, transport_override: Option<Transport>) -> SystemEvent {
    // Firmware chooses/display unit; send bare 'temp'
    let payload = "temp".to_string();
    match transport_override.unwrap_or(*state.transport.read().await) {
        Transport::Serial => forward_to_arduino(&payload, state).await,
        Transport::Mqtt => {
            let serial = state.serial.read().await;
            let topic = component_topic_sync(&serial, "temp");
            drop(serial);
            let mqtt = state.mqtt.read().await;
            match mqtt.publish(&topic, payload.as_bytes()).await {
                Ok(_) => SystemEvent::Output { content: format!("MQTT: sent {} - {} - ok", topic, payload) },
                Err(e) => SystemEvent::Error { source: "mqtt".into(), message: e },
            }
        },
    }
}

async fn exec_distance(args: &[&str], state: &AppState, transport_override: Option<Transport>) -> SystemEvent {
    let payload = if let Some(id) = args.get(0) { format!("distance {}", id) } else { "distance".to_string() };
    match transport_override.unwrap_or(*state.transport.read().await) {
        Transport::Serial => forward_to_arduino(&payload, state).await,
        Transport::Mqtt => {
            let serial = state.serial.read().await;
            let topic = component_topic_sync(&serial, "distance");
            drop(serial);
            let mqtt = state.mqtt.read().await;
            match mqtt.publish(&topic, payload.as_bytes()).await {
                Ok(_) => SystemEvent::Output { content: format!("MQTT: sent {} - {} - ok", topic, payload) },
                Err(e) => SystemEvent::Error { source: "mqtt".into(), message: e },
            }
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
        Transport::Mqtt => {
            let serial = state.serial.read().await;
            let topic = component_topic_sync(&serial, "led");
            drop(serial);
            let mqtt = state.mqtt.read().await;
            match mqtt.publish(&topic, payload.as_bytes()).await {
                Ok(_) => SystemEvent::Output { content: format!("MQTT: sent {} - {} - ok", topic, payload) },
                Err(e) => SystemEvent::Error { source: "mqtt".into(), message: e },
            }
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
            // Parse quoted lines: lcd show "line1" ["line2"]
            let joined = args[1..].join(" ");
            match parse_lcd_show_args(&joined) {
                Ok((l1, l2)) => {
                    // Optional length cap for 16x2 LCDs
                    let line1 = if l1.len() > 16 { l1[..16].to_string() } else { l1 };
                    let payload = if let Some(mut l2s) = l2 {
                        if l2s.len() > 16 { l2s = l2s[..16].to_string(); }
                        format!("lcd show \"{}\" \"{}\"", line1, l2s)
                    } else {
                        format!("lcd show \"{}\"", line1)
                    };
                    match transport_override.unwrap_or(*state.transport.read().await) {
                        Transport::Serial => forward_to_arduino(&payload, state).await,
                        Transport::Mqtt => {
                            let serial = state.serial.read().await;
                            let topic = component_topic_sync(&serial, "lcd");
                            drop(serial);
                            let mqtt = state.mqtt.read().await;
                            match mqtt.publish(&topic, payload.as_bytes()).await {
                                Ok(_) => SystemEvent::Output { content: format!("MQTT: sent {} - {} - ok", topic, payload) },
                                Err(e) => SystemEvent::Error { source: "mqtt".into(), message: e },
                            }
                        },
                    }
                }
                Err(msg) => SystemEvent::Error { source: "cli".into(), message: msg },
            }
        }
        _ => SystemEvent::Error { source: "cli".into(), message: "Usage: lcd <clear|show \"a\" [\"b\"]>".into() },
    }
}

fn parse_lcd_show_args(input: &str) -> Result<(String, Option<String>), String> {
    // Expect one or two quoted segments
    let bytes = input.as_bytes();
    let mut parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        // skip spaces
        while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
        if i >= bytes.len() { break; }
        if bytes[i] != b'"' { return Err("Usage: lcd show \"a\" [\"b\"]".into()); }
        i += 1; // skip opening quote
        let start = i;
        while i < bytes.len() && bytes[i] != b'"' { i += 1; }
        if i >= bytes.len() { return Err("Missing closing quote".into()); }
        let seg = &input[start..i];
        parts.push(seg.to_string());
        i += 1; // skip closing quote
    }
    if parts.is_empty() || parts.len() > 2 { return Err("Usage: lcd show \"a\" [\"b\"]".into()); }
    let first = parts[0].clone();
    let second = if parts.len() == 2 { Some(parts[1].clone()) } else { None };
    Ok((first, second))
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
                
                // On macOS prefer /dev/cu.* over /dev/tty.* and add a short retry loop for busy ports
                let mut last_err: Option<String> = None;
                let candidates: Vec<String> = if port_info.port_name.contains("/tty.") {
                    vec![port_info.port_name.replace("/tty.", "/cu."), port_info.port_name.clone()]
                } else {
                    vec![port_info.port_name.clone()]
                };

                let mut connected = false;
                for cand in &candidates {
                    // up to 3 attempts in case the port is busy right after upload/reset
                    for _attempt in 1..=3 {
                        let result = {
                            let mut serial = state.serial.write().await;
                            serial.connect(cand, baud, port_info.board_name.clone())
                        };
                        match result {
                            Ok(_) => { connected = true; break; }
                            Err(e) => {
                                last_err = Some(e.clone());
                                let lower = e.to_lowercase();
                                if lower.contains("busy") || lower.contains("device") || lower.contains("resource") {
                                    // small backoff then retry
                                    tokio::time::sleep(std::time::Duration::from_millis(700)).await;
                                    continue;
                                } else {
                                    break; // non-busy error, stop retrying this candidate
                                }
                            }
                        }
                    }
                    if connected { break; }
                }

                if connected {
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
                } else {
                    let msg = if let Some(e) = last_err {
                        let el = e.to_lowercase();
                        if el.contains("busy") || el.contains("resource busy") || el.contains("device busy") {
                            "Port is busy. Close Arduino IDE Serial Monitor/Plotter or any tool holding the port (screen, platformio, etc.). On macOS, try: lsof /dev/cu.* to see holders. After upload, re-open the web app and run 'ports' again; if needed, unplug/replug the USB to re-enumerate.".to_string()
                        } else {
                            e
                        }
                    } else {
                        "Failed to open port (unknown error)".to_string()
                    };
                    SystemEvent::Error { source: "serial".to_string(), message: msg }
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
    // If current transport is MQTT, publish to per-component topic and return
    if let Transport::Mqtt = *state.transport.read().await {
        match publish_component_command(state, "info", "info").await {
            Ok(_) => return SystemEvent::Output { content: "MQTT: info requested".into() },
            Err(e) => return SystemEvent::Error { source: "mqtt".into(), message: e },
        }
    }
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

    // Read multiple lines for up to ~3s, accept SENSORS line wherever it appears
    let mut sensors: Option<Vec<SensorDetail>> = None;
    let mut attempts = 0;
    let start = std::time::Instant::now();
    while start.elapsed().as_millis() < 3000 {
        match serial.read_line(400) {
            Ok(line) => {
                let line = line.trim();
                if line.starts_with("SENSORS:") {
                    let sensor_data = line.strip_prefix("SENSORS:").unwrap_or("");
                    let s: Vec<SensorDetail> = sensor_data
                        .split(',')
                        .enumerate()
                        .filter_map(|(i, part)| {
                            let kv: Vec<&str> = part.split(':').collect();
                            if kv.len() == 2 {
                                Some(SensorDetail { id: (i+1) as u8, name: kv[0].trim().to_string(), pin: format!("Pin {}", kv[1].trim()) })
                            } else { None }
                        })
                        .collect();
                    sensors = Some(s);
                    break;
                }
                // Ignore other banner lines
            }
            Err(_) => {
                attempts += 1;
                if attempts == 2 {
                    let _ = serial.send_command("/INFO");
                }
            }
        }
    }

    if let Some(sensors) = sensors {
        SystemEvent::SensorInfo {
            sensors,
            board: serial.get_board_name().unwrap_or("Unknown").to_string(),
            firmware: "v1.0.1".to_string(),
        }
    } else {
        SystemEvent::Output { content: "No sensor info returned from board (timeout). If sensors aren't connected, that's ok. Use 'status' or try commands like 'light on' or 'temp'.".to_string() }
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

// removed legacy helpers (forward_via_transport, handle_read, handle_led) as part of cleanup
