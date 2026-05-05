use std::time::Duration;

use actix_web::{
    HttpResponse, post,
    web::{Data, Json},
};
use lapin::{Channel, types::ShortString};
use log::{debug, error, info};
use serde_json::json;
use tokio::{sync::mpsc, time::sleep};

use crate::{
    PROJECT_CONFIG,
    api::schemas::{IncomingRequest, Request, ServiceResponse},
    configs::RESPONSE_CHANNELS,
    rmq::handler::send_message,
};

#[post("/request-servicehub")]
async fn send_and_receive(
    project_channel: Data<Channel>,
    request: Json<IncomingRequest>,
) -> HttpResponse {
    info!(
        "Processing application_id {}",
        request.application.application_id
    );
    let incoming_request: Request = Request::from(request.into_inner());
    let correlation_id = uuid::Uuid::new_v4().to_string();
    let (tx, mut rx) = mpsc::channel::<ServiceResponse>(2);

    let tx_queue = tx.clone();
    RESPONSE_CHANNELS
        .lock()
        .await
        .insert(ShortString::from(correlation_id.clone()), tx_queue);

    match send_message(&project_channel, &incoming_request, &correlation_id).await {
        Ok(()) => (),
        Err(e) => {
            error!("{e}");
            return HttpResponse::InternalServerError()
                .json(json!({"error": e.to_string()}));
        }
    };

    let tx_timeout = tx.clone();
    let correlation_id_timeout = correlation_id.clone();
    tokio::spawn(async move {
        let timeout_response = ServiceResponse::from(incoming_request);
        sleep(Duration::from_secs(
            PROJECT_CONFIG.servicehub_timeout as u64,
        ))
        .await;
        match tx_timeout.send(timeout_response).await {
            Ok(()) => {
                debug!("Sending Timeout message");
                RESPONSE_CHANNELS
                    .lock()
                    .await
                    .remove(&ShortString::from(correlation_id_timeout));
            }
            Err(e) => {
                debug!("Timeout message not send: {e}");
            }
        };
    });

    if let Some(response) = rx.recv().await {
        debug!("Received response {response:?}");
        drop(rx);
        drop(tx);
        info!("Dropped channel");
        debug!("Checking hashmap {:?}", RESPONSE_CHANNELS);
        return HttpResponse::Ok()
            .content_type("application/json")
            .json(serde_json::to_string(&response).unwrap());
    };

    HttpResponse::InternalServerError()
        .json(json!({"ServerError": "Tokio Channels error"}))
}
