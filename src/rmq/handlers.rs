//! Module for RMQ struct/function handlers
use lapin::options::BasicAckOptions;
use log::debug;
use log::warn;
use std::fmt::Debug;
use std::sync::Arc;
use tokio_stream::StreamExt;
use uuid::Uuid;

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
use crate::rmq::schemas::{Exchange, Queue};

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

    /// Method used to create a channel
    /// method is public so channel is used for every operation
    pub async fn create_channel(&mut self) -> Result<Channel, ProjectError> {
        info!("Creating channel");
        let channel = self.connection.create_channel().await.map_err(|e| {
            ProjectError::RmqError(RmqErrors::RMQChannelCreationError(e.to_string()))
        })?;
        Ok(channel)
    }

    /// Method to close channel + connection after shutdown
    pub async fn close(
        &self,
        channel: Arc<Channel>,
    ) -> () {
        info!("Shutting down consumer");
        channel.close(200, "Closing channel".into()).await.unwrap();
        self.connection
            .close(200, "Closing connection".into())
            .await
            .unwrap();
    }

    /// Main method that is used to consume queue waiting for response from core.
    /// When message is delivered, we check if correlation_id key is present in
    /// static RESPONSE_CHANNELS.
    /// If so, the value for that key is a mspc::Sender and the response is sent further
    /// The key is also deleted for duplication reasons.
    /// In case key not present in RESPONSE_CHANNELS => message consumed.
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
                format!("json_adapter_consumer_{}", Uuid::new_v4()).into(),
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| {
                ProjectError::RmqError(RmqErrors::ConsumerError(e.to_string()))
            })?;

        info!("Connection state {:?}", self.connection.status());
        while let Some(delivery_result) = consumer.next().await {
            let deliver = match delivery_result {
                Ok(deliver) => deliver,
                Err(e) => {
                    warn!("RabbitMQ Delivery Error: {e}");
                    continue;
                }
            };
            let correlation_id =
                match deliver.properties.correlation_id().as_ref().cloned() {
                    Some(id) => id,
                    None => {
                        warn!("Received response without correlation id");
                        deliver.ack(BasicAckOptions::default()).await.map_err(|e| {
                            ProjectError::RmqError(RmqErrors::RMQAckError(
                                e.to_string(),
                            ))
                        })?;
                        continue;
                    }
                };

            let service_response =
                match ServiceResponse::try_from(deliver.data.as_slice()) {
                    Ok(response) => response,
                    Err(e) => {
                        warn!(
                            "Invalid response for correlation ID {}: {}",
                            correlation_id, e
                        );
                        deliver.ack(BasicAckOptions::default()).await.map_err(|e| {
                            ProjectError::RmqError(RmqErrors::RMQAckError(
                                e.to_string(),
                            ))
                        })?;
                        continue;
                    }
                };

            let sender = RESPONSE_CHANNELS.lock().await.remove(&correlation_id);
            match sender {
                Some(sender) => {
                    if sender.send(service_response).is_err() {
                        debug!(
                            "HTTP receiver closed before response arrived: {}",
                            correlation_id
                        );
                    }
                }
                None => {
                    debug!(
                        "No pending request for correlation ID {}; \
                                 likely timed out, disconnected, duplicated, \
                                 restarted, or delivered to another instance",
                        correlation_id
                    );
                }
            };
            deliver.ack(BasicAckOptions::default()).await.map_err(|e| {
                ProjectError::RmqError(RmqErrors::RMQAckError(e.to_string()))
            })?;
        }

        Ok(())
    }
}

/// Function to send message through RabbitMQ.
/// Mainly used for core/adapter communications.
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
            &serde_json::to_vec(&data)
                .map_err(|e| RmqErrors::RMQPublishError(e.to_string()))?,
            BasicProperties::default().with_correlation_id(correlation_id.into()),
        )
        .await
        .map_err(|e| RmqErrors::RMQPublishError(e.to_string()))?;
    Ok(())
}
