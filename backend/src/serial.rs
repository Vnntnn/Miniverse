use crate::models::SerialPort;
use anyhow::{Result, Context};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use std::time::Duration;

pub struct SerialManager {
    current_port: Option<SerialStream>,
    port_name: Option<String>,
}

impl SerialManager {
    pub fn new() -> Self {
        Self {
            current_port: None,
            port_name: None,
        }
    }

    pub fn list_ports() -> Result<Vec<SerialPort>> {
        let ports = tokio_serial::available_ports()
            .context("Failed to enumerate serial ports")?;

        let mut serial_ports = Vec::new();
        
        for port in ports {
            let serial_port = SerialPort {
                port_name: port.port_name.clone(),
                port_type: match port.port_type {
                    tokio_serial::SerialPortType::UsbPort(_) => "USB".to_string(),
                    tokio_serial::SerialPortType::BluetoothPort => "Bluetooth".to_string(),
                    tokio_serial::SerialPortType::PciPort => "PCI".to_string(),
                    tokio_serial::SerialPortType::Unknown => "Unknown".to_string(),
                },
                description: match &port.port_type {
                    tokio_serial::SerialPortType::UsbPort(info) => info.product.clone(),
                    _ => None,
                },
                manufacturer: match &port.port_type {
                    tokio_serial::SerialPortType::UsbPort(info) => info.manufacturer.clone(),
                    _ => None,
                },
                product: match &port.port_type {
                    tokio_serial::SerialPortType::UsbPort(info) => info.product.clone(),
                    _ => None,
                },
                vendor_id: match &port.port_type {
                    tokio_serial::SerialPortType::UsbPort(info) => Some(info.vid),
                    _ => None,
                },
                product_id: match &port.port_type {
                    tokio_serial::SerialPortType::UsbPort(info) => Some(info.pid),
                    _ => None,
                },
            };
            serial_ports.push(serial_port);
        }

        Ok(serial_ports)
    }

    pub async fn connect(&mut self, port_name: &str, baud_rate: u32) -> Result<()> {
        let port = tokio_serial::new(port_name, baud_rate)
            .timeout(Duration::from_millis(1000))
            .open_native_async()
            .context("Failed to open serial port")?;

        self.current_port = Some(port);
        self.port_name = Some(port_name.to_string());

        log::info!("Connected to {} at {} baud", port_name, baud_rate);
        Ok(())
    }

    pub fn disconnect(&mut self) {
        if let Some(port_name) = &self.port_name {
            log::info!("Disconnected from {}", port_name);
        }
        self.current_port = None;
        self.port_name = None;
    }

    pub fn is_connected(&self) -> bool {
        self.current_port.is_some()
    }

    pub fn get_connected_port(&self) -> Option<&str> {
        self.port_name.as_deref()
    }
}
