use crate::errors::{ProjectError, rmq::RmqErrors};
use crate::rmq::handler::RmqHandler;
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

    async fn build_rmq_connection(&self) -> Result<Connection, ProjectError> {
        let rmq_url = &self.rmq_url;
        Connection::connect(
            rmq_url,
            ConnectionProperties::default().enable_auto_recover(),
        )
        .await
        .map_err(|e| {
            ProjectError::RmqError(RmqErrors::RMQConnectionError(e.to_string()))
        })
    }

    pub async fn build(self) -> Result<RmqHandler, ProjectError> {
        info!("---- Starting RMQ connection ----");
        let rmq_connection = self.build_rmq_connection().await?;
        info!("---- RMQ Connection established ----");

        Ok(RmqHandler::new(rmq_connection))
    }
}
