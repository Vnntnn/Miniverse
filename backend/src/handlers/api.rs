use actix_web::{HttpResponse, Result};
use serde::Serialize;

#[derive(Serialize)]
pub struct PortInfo {
    pub port_name: String,
    pub port_type: String,
}

pub async fn get_available_ports() -> Result<HttpResponse> {
    match tokio_serial::available_ports() {
        Ok(ports) => {
            let port_list: Vec<PortInfo> = ports
                .into_iter()
                .map(|p| PortInfo {
                    port_name: p.port_name,
                    port_type: format!("{:?}", p.port_type),
                })
                .collect();

            Ok(HttpResponse::Ok().json(port_list))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().body(format!("Error: {}", e))),
    }
}
