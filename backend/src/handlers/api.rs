use actix_web::{HttpResponse, Result};
use crate::serial::SerialManager;

pub async fn get_available_ports() -> Result<HttpResponse> {
    match SerialManager::list_ports() {
        Ok(ports) => Ok(HttpResponse::Ok().json(ports)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(format!("Error: {}", e))),
    }
}
