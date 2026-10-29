use actix_web::{
    App, HttpServer,
    dev::ServerHandle,
    main, middleware,
    web::{Data, scope},
};
use envconfig::Envconfig;
use log::{error, info, warn};
use std::{
    collections::HashMap,
    io::Result,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{signal, sync::Mutex};
use uuid::Uuid;

use actix_lapin::{
    api::{base::send_and_receive, ping::healthcheck, schemas::RMQTarget},
    configs::Config,
    rmq::builder::ConnectionBuilder,
    state::AppState,
};
use dotenvy::dotenv;
use env_logger::{Env, init_from_env};

#[main]
async fn main() -> Result<()> {
    dotenv().ok();
    init_from_env(Env::default().default_filter_or("info"));

    // Lazy project configs
    let project_configs: Config = Config::init_from_env().expect("Failed to load envs");

    let rmq_connection_builder =
        ConnectionBuilder::new().with_rmq_url(project_configs.get_rmq_url());

    let mut rmq_handler = rmq_connection_builder
        .build()
        .await
        .expect("failed to connect to RabbitMQ");

    let channel = Arc::new(
        rmq_handler
            .create_channel()
            .await
            .expect("failed to create RabbitMQ channel"),
    );

    let response_exchage = project_configs.rmq_exchange.clone();
    let response_queue =
        format!("{}.{}", project_configs.rmq_response_queue, Uuid::new_v4());
    let rmq_target = RMQTarget::new(&project_configs, response_queue.clone());
    let app_state = Arc::new(AppState {
        project_configs,
        rmq_channel: channel.clone(),
        rmq_target,
        shutdown_state: AtomicBool::new(false),
        response_channels: Arc::new(Mutex::new(HashMap::new())),
    });

    let http_state = Data::new(app_state.clone());

    let server = HttpServer::new(move || {
        App::new()
            .app_data(http_state.clone())
            .wrap(middleware::Logger::default())
            .service(
                scope("/api/v1")
                    .service(healthcheck)
                    .service(send_and_receive),
            )
    })
    .bind(("0.0.0.0", 8000))?
    .workers(1)
    .shutdown_timeout(30)
    .keep_alive(Duration::from_secs(75))
    .client_request_timeout(Duration::from_secs(60))
    .disable_signals()
    .run();

    let server_handle = server.handle();
    let shutdown_state = app_state.clone();
    let shutdown_channel = channel.clone();

    let lifecycle_task = tokio::spawn(async move {
        tokio::select! {
            consume_result = rmq_handler.consume_main(&response_queue, &response_exchage, &app_state.response_channels) => {
                match consume_result {
                    Ok(()) => {
                        warn!("RabbitMQ consumer stopped unexpectedly");
                    }
                    Err(error) => {
                        error!("RabbitMQ consumer error: {error:?}");
                    }
                }

                shutdown_application(&shutdown_state, &server_handle).await;
            }

            signal_result = shutdown_signal() => {
                match signal_result {
                    Ok(signal_name) => {
                        info!("Received {signal_name}");
                    }
                    Err(error) => {
                        error!("Failed to listen for shutdown signal: {error}");
                    }
                }

                shutdown_application(
                    &shutdown_state,
                    &server_handle,
                ).await;
            }
        }

        rmq_handler.close(shutdown_channel).await;
    });

    let server_result = server.await;
    if let Err(join_error) = lifecycle_task.await {
        error!("Lifecycle task failed: {join_error}");
    }

    server_result
}

async fn shutdown_application(
    shutdown_state: &AppState,
    server_handle: &ServerHandle,
) {
    shutdown_state.shutdown_state.store(true, Ordering::SeqCst);

    info!("Marked application unhealthy; waiting for load balancer drain");
    tokio::time::sleep(Duration::from_secs(5)).await;

    info!("Stopping HTTP server gracefully");
    server_handle.stop(true).await;
}

#[cfg(unix)]
async fn shutdown_signal() -> std::io::Result<&'static str> {
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())?;

    tokio::select! {
        result = signal::ctrl_c() => {
            result?;
            Ok("SIGINT")
        }
        _ = sigterm.recv() => {
            Ok("SIGTERM")
        }
    }
}

#[cfg(not(unix))]
async fn shutdown_signal() -> std::io::Result<&'static str> {
    signal::ctrl_c().await?;
    Ok("Ctrl+C")
}
