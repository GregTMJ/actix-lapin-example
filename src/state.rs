use std::sync::{Arc, atomic::AtomicBool};

use lapin::Channel;

pub struct AppState {
    pub rmq_channel: Arc<Channel>,
    pub shutdown_state: AtomicBool,
}
