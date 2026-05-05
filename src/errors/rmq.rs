use thiserror::Error;

#[derive(Debug, Default, Error)]
pub enum RmqErrors {
    #[error("Error while connecting to RMQ: {0}")]
    RMQConnectionError(String),
    #[error("Channel on create error: {0}")]
    RMQChannelCreationError(String),
    #[error("Channel method error: {0}")]
    RMQChannelError(String),
    #[error("Message publish error: {0}")]
    RMQPublishError(String),
    #[error("{0}")]
    RMQAckError(String),
    #[error("Failed to consume: {0}")]
    ConsumerError(String),
    #[error("Error on sending result")]
    ResponseSenderError,
    #[error("Unkown RMQ error")]
    #[default]
    Unkown,
}
