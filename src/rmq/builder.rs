//! Module to build a connection for RabbitMQ.
use crate::errors::{ProjectError, rmq::RmqErrors};
use crate::rmq::handlers::RmqHandler;
use lapin::{Connection, ConnectionProperties};
use log::info;

#[derive(Default)]
pub struct ConnectionBuilder {
    rmq_url: String,
}

impl ConnectionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rmq_url(
        mut self,
        url: String,
    ) -> Self {
        self.rmq_url = url;
        self
    }

    /// Method starts a connection with RabbitMQ and returns it
    /// if no error acquires.
    pub async fn build(self) -> Result<RmqHandler, ProjectError> {
        info!("---- Starting RMQ connection ----");
        let rmq_connection = Connection::connect(
            &self.rmq_url,
            ConnectionProperties::default().enable_auto_recover(),
        )
        .await
        .map_err(|e| {
            ProjectError::RmqError(RmqErrors::RMQConnectionError(e.to_string()))
        })?;
        info!("---- RMQ Connection established ----");

        Ok(RmqHandler::new(rmq_connection))
    }
}
