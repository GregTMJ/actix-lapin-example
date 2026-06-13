//! Module for API errors.
use thiserror::Error;

#[derive(Debug, Error, Default)]
pub enum ApiErrors {
    #[error("DeserializeError: {0}")]
    DeserializeResponseError(String),
    #[error("UnknownError")]
    #[default]
    Unknown,
}
