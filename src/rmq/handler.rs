use lapin::options::BasicAckOptions;
use lapin::types::ShortString;
use log::debug;
use log::warn;
use std::fmt::Debug;
use std::sync::Arc;
use tokio_stream::StreamExt;

use crate::api::schemas::Request;
use crate::api::schemas::ServiceResponse;
use crate::configs::RESPONSE_CHANNELS;
use crate::errors::ProjectError;
use lapin::BasicProperties;
use lapin::Channel;
use lapin::Connection as AMQPConnection;
use lapin::options::BasicConsumeOptions;
use lapin::options::BasicPublishOptions;
use lapin::options::{ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use log::info;

use crate::PROJECT_CONFIG;
use crate::errors::rmq::RmqErrors;

use lapin::ExchangeKind;

#[derive(Debug)]
pub struct Exchange<'a> {
    pub name: &'a str,
    pub exchange_type: ExchangeKind,
}

impl<'a> Exchange<'a> {
    pub fn new(
        name: &'a str,
        exchange_type: &str,
    ) -> Self {
        let lowercased_exchange_type = exchange_type.to_ascii_lowercase();
        let exchange_type = match lowercased_exchange_type.as_str() {
            "direct" => ExchangeKind::Direct,
            "fanout" => ExchangeKind::Fanout,
            "headers" => ExchangeKind::Headers,
            _ => ExchangeKind::Topic,
        };
        Self {
            name,
            exchange_type,
        }
    }
}

#[derive(Debug, Default)]
pub struct Queue<'a> {
    pub name: &'a str,
    pub routing_key: &'a str,
}

impl<'a> Queue<'a> {
    pub fn new(
        name: &'a str,
        routing_key: &'a str,
    ) -> Self {
        Self { name, routing_key }
    }
}

#[derive(Debug)]
pub struct RmqHandler {
    connection: Arc<AMQPConnection>,
}

impl RmqHandler {
    pub fn new(connection: AMQPConnection) -> Self {
        Self {
            connection: Arc::new(connection),
        }
    }

    pub async fn create_channel(&mut self) -> Result<Channel, ProjectError> {
        info!("Creating channel");
        let channel = self.connection.create_channel().await.map_err(|e| {
            ProjectError::RmqError(RmqErrors::RMQChannelCreationError(e.to_string()))
        })?;
        Ok(channel)
    }

    pub async fn consume_main(&mut self) -> Result<(), ProjectError> {
        let current_channel = self.create_channel().await?;
        let exchange = Exchange::new(&PROJECT_CONFIG.rmq_exchange, "direct");
        let queue = Queue::new(
            &PROJECT_CONFIG.rmq_response_queue,
            &PROJECT_CONFIG.rmq_response_queue,
        );
        info!("Binding queue {} to channel", queue.name);
        current_channel
            .exchange_declare(
                exchange.name.into(),
                exchange.exchange_type.clone(),
                ExchangeDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|msg| {
                ProjectError::RmqError(RmqErrors::RMQChannelError(msg.to_string()))
            })?;
        current_channel
            .queue_declare(
                queue.name.into(),
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await
            .map_err(|msg| {
                ProjectError::RmqError(RmqErrors::RMQChannelError(msg.to_string()))
            })?;
        current_channel
            .queue_bind(
                queue.name.into(),
                exchange.name.into(),
                queue.routing_key.into(),
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|msg| {
                ProjectError::RmqError(RmqErrors::RMQChannelError(msg.to_string()))
            })?;
        info!("Binding queue {} successful", queue.name);

        let mut consumer = current_channel
            .basic_consume(
                queue.name.into(),
                "json_adapter_consumer".into(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| {
                ProjectError::RmqError(RmqErrors::ConsumerError(e.to_string()))
            })?;

        info!("Connection state {:?}", self.connection.status());
        while let Some(delivery) = consumer.next().await {
            if let Ok(delivery) = delivery {
                delivery
                    .ack(BasicAckOptions::default())
                    .await
                    .map_err(|e| {
                        ProjectError::RmqError(RmqErrors::RMQAckError(e.to_string()))
                    })?;
                let serhub_response: ServiceResponse =
                    ServiceResponse::try_from(delivery.data).map_err(|e| {
                        ProjectError::RmqError(RmqErrors::RMQAckError(e.to_string()))
                    })?;
                debug!("Checking hashmap on consumer {:?}", RESPONSE_CHANNELS);
                match RESPONSE_CHANNELS.lock().await.remove(
                    delivery
                        .properties
                        .correlation_id()
                        .as_ref()
                        .unwrap_or(&ShortString::from("default")),
                ) {
                    Some(sender) => {
                        if let Err(e) = sender.send(serhub_response).await {
                            debug!("Receiver already closed {e}")
                        }
                    }
                    None => {
                        warn!(
                            "Received unexpected message for application_id {}",
                            serhub_response.application_id,
                        )
                    }
                }
            }
        }

        std::future::pending::<()>().await;
        Ok(())
    }
}

pub async fn send_message(
    channel: &Arc<Channel>,
    data: &Request,
    correlation_id: &str,
) -> Result<(), RmqErrors> {
    channel
        .basic_publish(
            PROJECT_CONFIG.rmq_servicehub_exchange.as_str().into(),
            PROJECT_CONFIG.rmq_servicehub_request_queue.as_str().into(),
            BasicPublishOptions::default(),
            &serde_json::to_vec(&data).unwrap(),
            BasicProperties::default().with_correlation_id(correlation_id.into()),
        )
        .await
        .map_err(|e| RmqErrors::RMQPublishError(e.to_string()))?;
    Ok(())
}
