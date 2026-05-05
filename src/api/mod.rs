use std::sync::Arc;

use crate::rmq::handler::RmqHandler;

pub mod base;
pub mod ping;
pub mod schemas;

pub struct ApiState {
    pub rmq_handler: Arc<RmqHandler>,
}
