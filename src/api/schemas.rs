use std::time::SystemTime;

use crate::{errors::api::ApiErrors, prelude::PROJECT_CONFIG};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Application {
    pub application_id: Uuid,
    pub system_id: u16,
    pub service_id: u16,
    pub request_system_name: Option<String>,
    pub is_resend: bool,
    pub multi_request: bool,
}

impl Default for Application {
    fn default() -> Self {
        Application {
            application_id: Uuid::new_v4(),
            system_id: 1,
            service_id: 1,
            request_system_name: None,
            is_resend: false,
            multi_request: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RMQTarget {
    pub vhost: String,
    pub exchange: String,
    pub routing_key: String,
}

impl Default for RMQTarget {
    fn default() -> Self {
        Self {
            vhost: PROJECT_CONFIG.rmq_vhost.clone(),
            exchange: PROJECT_CONFIG.rmq_exchange.clone(),
            routing_key: PROJECT_CONFIG.rmq_response_queue.clone(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Request {
    pub application: Application,
    pub target: RMQTarget,
    pub person: Map<String, Value>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IncomingRequest {
    pub application: Application,
    person: Map<String, Value>,
}

impl From<IncomingRequest> for Request {
    fn from(value: IncomingRequest) -> Self {
        Self {
            application: value.application,
            person: value.person,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub application_id: Uuid,
    serhub_request_id: Option<Uuid>,
    service_id: u16,
    system_id: u16,
    is_cache: bool,
    status: String,
    status_description: Value,
    request_system_name: Option<String>,
    response_created_time: String,
    response: Option<Value>,
}

impl TryFrom<Vec<u8>> for ServiceResponse {
    type Error = ApiErrors;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let response = serde_json::from_slice(&value)
            .map_err(|e| ApiErrors::DeserializeResponseError(e.to_string()))?;
        Ok(response)
    }
}

impl From<Request> for ServiceResponse {
    fn from(value: Request) -> Self {
        let now = SystemTime::now();
        let chrono_time: DateTime<Utc> = now.into();
        Self {
            application_id: value.application.application_id,
            serhub_request_id: None,
            service_id: value.application.service_id,
            system_id: value.application.system_id,
            is_cache: false,
            status: "ServiceTimeout".to_string(),
            status_description: json!("[]"),
            request_system_name: None,
            response_created_time: chrono_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            response: None,
        }
    }
}
