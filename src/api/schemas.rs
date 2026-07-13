//! Module for all available schemas that is used in AMQP proccess.
use std::time::SystemTime;

use crate::{errors::api::ApiErrors, prelude::PROJECT_CONFIG};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use uuid::Uuid;

/// Struct that contains meta data for requests.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Application {
    pub application_id: Uuid,
    pub system_id: u16,
    pub service_id: u16,
    pub request_system_name: Option<String>,
    pub is_resend: bool,
    pub multi_request: bool,
}

impl PartialEq for Application {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.service_id == other.service_id
    }
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

/// Struct for RabbitMQ meta data
#[derive(Debug, Serialize, Deserialize, PartialEq)]
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

/// Struct for common Request.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Request {
    pub application: Application,
    pub target: RMQTarget,
    // field, that stores request incoming data
    pub person: Map<String, Value>,
}

/// Basic struct without target when incoming message is parsed.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct IncomingRequest {
    pub application: Application,
    person: Map<String, Value>,
}

/// No need to use TryFrom because application & person are already validated
impl From<IncomingRequest> for Request {
    fn from(value: IncomingRequest) -> Self {
        Self {
            application: value.application,
            person: value.person,
            ..Default::default()
        }
    }
}

/// Struct used to validate incoming response from main process.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
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

impl TryFrom<&[u8]> for ServiceResponse {
    type Error = ApiErrors;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let response = serde_json::from_slice(value)
            .map_err(|e| ApiErrors::DeserializeResponseError(e.to_string()))?;
        Ok(response)
    }
}

/// Method to form a ServiceTimeout response.
impl From<Request> for ServiceResponse {
    fn from(value: Request) -> Self {
        let chrono_time: DateTime<Utc> = SystemTime::now().into();
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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_default_application() -> Application {
        Application::default()
    }

    fn get_default_rmq_target() -> RMQTarget {
        RMQTarget::default()
    }

    fn get_default_request() -> Request {
        Request::default()
    }

    fn get_default_incoming_request() -> IncomingRequest {
        IncomingRequest::default()
    }

    #[test]
    fn test_application() {
        let default_application = get_default_application();
        assert_eq!(default_application.service_id, 1);
        assert_eq!(default_application.system_id, 1);
        assert_eq!(default_application.multi_request, false);
    }

    #[test]
    fn test_rmq_target() {
        let default_target = get_default_rmq_target();
        assert_eq!(
            default_target.routing_key,
            PROJECT_CONFIG.rmq_response_queue.clone()
        )
    }

    #[test]
    fn test_request() {
        let default_request = get_default_request();
        assert_eq!(default_request.person, Map::new());
        assert_eq!(default_request.application, get_default_application());
        assert_eq!(default_request.target, get_default_rmq_target());
    }

    #[test]
    fn test_incoming_request() {
        let default_request = get_default_incoming_request();
        assert_eq!(default_request.person, Map::new());
        assert_eq!(default_request.application, get_default_application());
    }

    #[test]
    fn test_from_incoming_request() {
        let incoming_request = get_default_incoming_request();
        let request = Request::from(incoming_request.clone());
        assert_eq!(request.application, incoming_request.application);
    }

    #[test]
    fn test_service_response_ok() {
        let incoming_service_response = ServiceResponse::default();
        let response_as_vec = serde_json::to_vec(&incoming_service_response).unwrap();
        let service_response = ServiceResponse::try_from(response_as_vec.as_slice());
        assert!(service_response.is_ok());
        let response = service_response.unwrap();
        assert_eq!(response.service_id, 0);
        assert_eq!(response.response, None);
    }

    #[test]
    fn test_service_response_fail() {
        let incoming_service_response = "Test data".to_string();
        let response_as_vec = serde_json::to_vec(&incoming_service_response).unwrap();
        let service_response: Result<ServiceResponse, ApiErrors> =
            response_as_vec.as_slice().try_into();
        assert!(service_response.is_err());
    }

    #[test]
    fn test_service_response_from_request() {
        let default_request = get_default_request();
        let service_response_timeout: ServiceResponse = default_request.into();
        assert_eq!(service_response_timeout.is_cache, false);
        assert_eq!(
            service_response_timeout.status,
            "ServiceTimeout".to_string()
        );
        assert!(service_response_timeout.response.is_none());
    }
}
