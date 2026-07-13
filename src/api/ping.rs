//! Simple HTTP ping for server/RabbitMQ check
//! Module to ping the server checking if Lapin is still connected to RabbitMQ.

use std::sync::atomic::Ordering;

use actix_web::{HttpResponse, Responder, get, web::Data};
use log::debug;
use serde_json::json;

use crate::state::AppState;

#[get("/ping")]
async fn healthcheck(project_state: Data<AppState>) -> impl Responder {
    debug!("Pinging server");
    if project_state.shutdown_state.load(Ordering::SeqCst) {
        return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "shutting_down"
        }));
    }
    if project_state.rmq_channel.status().connected() {
        HttpResponse::Ok().json(json!({"success": true}))
    } else {
        HttpResponse::ServiceUnavailable().json(json!({"success": false}))
    }
}
