//! Simple HTTP ping for server/RabbitMQ check
//! Module to ping the server checking if Lapin is still connected to RabbitMQ.
use actix_web::{HttpResponse, Responder, get, web::Data};
use lapin::Channel;
use serde_json::json;

#[get("/ping")]
async fn healthcheck(channel: Data<Channel>) -> impl Responder {
    if channel.status().connected() {
        HttpResponse::Ok().json(json!({"success": true}))
    } else {
        HttpResponse::ServiceUnavailable().json(json!({"success": false}))
    }
}
