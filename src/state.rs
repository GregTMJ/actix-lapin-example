use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicBool},
};

use lapin::{Channel, types::ShortString};
use tokio::sync::{Mutex, oneshot};

use crate::{
    api::schemas::{RMQTarget, ServiceResponse},
    configs::Config,
};

pub type ResponseChannelMap =
    Mutex<HashMap<ShortString, oneshot::Sender<ServiceResponse>>>;

pub struct AppState {
    pub project_configs: Config,
    pub rmq_channel: Arc<Channel>,
    pub rmq_target: RMQTarget,
    pub shutdown_state: AtomicBool,
    pub response_channels: Arc<ResponseChannelMap>,
}
