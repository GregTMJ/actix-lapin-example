use crate::errors::{api::ApiErrors, rmq::RmqErrors};

pub mod api;
pub mod rmq;

#[derive(Debug)]
pub enum ProjectError {
    ApiError(ApiErrors),
    RmqError(RmqErrors),
}
