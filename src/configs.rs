use envconfig::Envconfig;
use serde::Deserialize;

/// Main config struct
#[allow(non_snake_case)]
#[derive(Debug, Default, Deserialize, Envconfig)]
pub struct Config {
    #[envconfig(from = "RMQ_USER", default = "user")]
    rmq_user: String,
    #[envconfig(from = "RMQ_PASSWORD", default = "bitnami")]
    rmq_password: String,
    #[envconfig(from = "RMQ_HOST", default = "rabbitmq")]
    rmq_host: String,
    #[envconfig(from = "RMQ_PORT", default = "5672")]
    rmq_port: u16,
    #[envconfig(from = "RMQ_VHOST", default = "%2f")]
    pub rmq_vhost: String,
    #[envconfig(from = "RMQ_PARAMS", default = "")]
    rmq_params: String,

    // service configs
    #[envconfig(from = "RMQ_EXCHANGE", default = "servicehub")]
    pub rmq_exchange: String,
    #[envconfig(from = "RMQ_RESPONSE_QUEUE", default = "json_adapter.q.response")]
    pub rmq_response_queue: String,

    #[envconfig(from = "RMQ_SERVICEHUB_EXCHANGE", default = "servicehub")]
    pub rmq_servicehub_exchange: String,
    #[envconfig(
        from = "RMQ_SERVICEHUB_REQUEST_QUEUE",
        default = "servicehub.q.request"
    )]
    pub rmq_servicehub_request_queue: String,

    #[envconfig(from = "SERVICEHUB_TIMEOUT", default = "30")]
    pub servicehub_timeout: u16,
}

impl Config {
    // Method to set rmq_url
    pub fn get_rmq_url(&self) -> String {
        format!(
            "amqp://{}:{}@{}:{}/{}{}",
            self.rmq_user,
            self.rmq_password,
            self.rmq_host,
            self.rmq_port,
            self.rmq_vhost,
            self.rmq_params
        )
    }
}
