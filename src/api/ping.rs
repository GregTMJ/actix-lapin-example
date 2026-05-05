use actix_web::{get, web::Data, HttpResponse, Responder};
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
