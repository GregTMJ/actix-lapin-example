use actix_web::web::scope;
use log::warn;
use std::io::Result;
use tokio::task;

use actix_lapin::api::base::send_and_receive;
use actix_lapin::api::ping::healthcheck;
use actix_lapin::{PROJECT_CONFIG, rmq::builder::ConnectionBuilder};
use actix_web::{App, HttpServer, main, middleware, web::Data};
use dotenvy::dotenv;
use env_logger::{Env, init_from_env};

#[main]
async fn main() -> Result<()> {
    dotenv().ok();
    init_from_env(Env::default().default_filter_or("info"));

    let rmq_connection_builder =
        ConnectionBuilder::new().with_rmq_url(PROJECT_CONFIG.get_rmq_url());

    let mut rmq_handler = rmq_connection_builder.build().await.unwrap();
    let channel = rmq_handler.create_channel().await.unwrap();

    task::spawn(async move {
        if let Err(e) = rmq_handler.consume_main().await {
            warn!("Consumer error: {:?}", e)
        }
    });

    let shared_channel = Data::new(channel);

    HttpServer::new(move || {
        App::new()
            .app_data(shared_channel.clone())
            .wrap(middleware::Logger::default())
            .service(
                scope("/api/v1")
                    .service(healthcheck)
                    .service(send_and_receive),
            )
    })
    .bind(("0.0.0.0", 8000))?
    .workers(2)
    .run()
    .await
}
