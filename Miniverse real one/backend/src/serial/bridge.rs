use serialport::{SerialPort, SerialPortType, UsbPortInfo};
use serde::{Serialize, Deserialize};
use serde::Serialize;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct SendableSerialPort(Box<dyn SerialPort>);
unsafe impl Send for SendableSerialPort {}

pub struct SerialBridge {
    port: Option<Arc<Mutex<SendableSerialPort>>>,
    port_name: Option<String>,
    board_name: Option<String>,
    baud_rate: u32,
}

impl SerialBridge {
    pub fn new() -> Self {
        Self {
            port: None,
            port_name: None,
            board_name: None,
            baud_rate: 9600,
        }
    }
    
    pub fn list_ports() -> Result<Vec<PortInfo>, String> {
        serialport::available_ports()
            .map(|ports| {
                ports
                    .iter()
                    .enumerate()
                    .map(|(idx, p)| {
                        let board_name = match &p.port_type {
                            SerialPortType::UsbPort(info) => Self::detect_board_name(info),
                            _ => "Unknown Device".to_string(),
                        };
                        
                        PortInfo {
                            index: idx,
                            port_name: p.port_name.clone(),
                            board_name,
                        }
                    })
                    .collect()
            })
            .map_err(|e| format!("List ports failed: {}", e))
    }
    
    fn detect_board_name(info: &UsbPortInfo) -> String {
        let vid = info.vid;
        let pid = info.pid;

        // Prefer explicit, human-friendly names for common boards
        let known = match (vid, pid) {
            // Official Arduino VID
            (0x2341, 0x0043) | (0x2341, 0x0001) => Some("Arduino Uno"),
            (0x2341, 0x0042) => Some("Arduino Mega 2560"),
            (0x2341, 0x8036) => Some("Arduino Leonardo"),
            (0x2341, 0x8037) => Some("Arduino Micro"),
            (0x2341, 0x0058) => Some("Arduino Nano 33 IoT"),
            (0x2341, 0x804d) => Some("Arduino/Genuino Zero"),
            (0x2341, 0x804e) => Some("Arduino/Genuino MKR1000"),

            // FTDI
            (0x0403, 0x6001) => Some("FTDI FT232R (Arduino Compatible)"),

            // Silicon Labs CP210x
            (0x10c4, 0xea60) => Some("CP2102 USB-UART (Arduino/ESP Compatible)"),

            // WCH CH34x
            (0x1a86, 0x7523) => Some("CH340 USB-Serial (Arduino Compatible)"),
            (0x1a86, 0x55d4) => Some("CH9102 USB-Serial (Arduino/ESP Compatible)"),

            // Espressif
            (0x303a, _) => Some("Espressif ESP32/ESP8266 (USB)"),

            _ => None,
        };

        if let Some(name) = known {
            return name.to_string();
        }

        // If product/manufacturer strings exist, compose a friendly name
        let product = info.product.as_deref().unwrap_or("");
        let manufacturer = info.manufacturer.as_deref().unwrap_or("");
        if !product.is_empty() || !manufacturer.is_empty() {
            let mut s = String::new();
            if !manufacturer.is_empty() { s.push_str(manufacturer); }
            if !product.is_empty() {
                if !s.is_empty() { s.push(' '); }
                s.push_str(product);
            }
            return s;
        }

        format!("USB Device (VID:{:04x} PID:{:04x})", vid, pid)
    }
    
    pub fn connect(&mut self, port_name: &str, baud_rate: u32, board_name: String) -> Result<(), String> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| format!("Open port failed: {}", e))?;
        
        self.port = Some(Arc::new(Mutex::new(SendableSerialPort(port))));
        self.port_name = Some(port_name.to_string());
        self.board_name = Some(board_name.clone());
        self.baud_rate = baud_rate;
        
        log::info!("Serial connected: {} ({}) @ {}", port_name, board_name, baud_rate);
        Ok(())
    }
    
    pub fn disconnect(&mut self) {
        self.port = None;
        self.port_name = None;
        self.board_name = None;
        log::info!("Serial disconnected");
    }
    
    pub fn is_connected(&self) -> bool {
        self.port.is_some()
    }
    
    pub fn get_port_name(&self) -> Option<&str> {
        self.port_name.as_deref()
    }
    
    pub fn get_board_name(&self) -> Option<&str> {
        self.board_name.as_deref()
    }
    
    pub fn get_baud_rate(&self) -> u32 {
        self.baud_rate
    }
    
    pub fn send_command(&self, cmd: &str) -> Result<(), String> {
        let port = self.port.as_ref().ok_or("Not connected")?;
        let mut port = port.lock().unwrap();
        port.0
            .write_all(format!("{}\n", cmd).as_bytes())
            .map_err(|e| format!("Write failed: {}", e))
    }
    
    pub fn read_line(&self, timeout_ms: u64) -> Result<String, String> {
        let port = self.port.as_ref().ok_or("Not connected")?;
        let mut port = port.lock().unwrap();
        let mut buffer = Vec::new();
        let mut byte = [0u8; 1];
        
        let start = std::time::Instant::now();
        
        loop {
            if start.elapsed().as_millis() > timeout_ms as u128 {
                return Err("Timeout".to_string());
            }
            
            match port.0.read(&mut byte) {
                Ok(1) => {
                    if byte[0] == b'\n' {
                        break;
                    }
                    if byte[0] != b'\r' {
                        buffer.push(byte[0]);
                    }
                }
                Ok(_) => continue,
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(e) => return Err(format!("Read error: {}", e)),
            }
        }
        
        String::from_utf8(buffer).map_err(|e| format!("UTF-8 error: {}", e))
    }
}

#[derive(Debug, Clone, Serialize)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub index: usize,
    pub port_name: String,
    pub board_name: String,
}
