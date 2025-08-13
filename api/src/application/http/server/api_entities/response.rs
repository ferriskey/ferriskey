use axum::http::StatusCode;
use axum::{Json, response::IntoResponse};
use serde::Serialize;

use super::api_success::ApiSuccess;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response<T: Serialize + PartialEq> {
    OK(T),
    Created(T),
    Accepted(T),
}

impl<T: Serialize + PartialEq> IntoResponse for Response<T> {
    fn into_response(self) -> axum::response::Response {
        match self {
            Response::OK(data) => (StatusCode::OK, Json(data)).into_response(),
            Response::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Response::Accepted(data) => (StatusCode::ACCEPTED, Json(data)).into_response(),
        }
    }
}

impl<T: Serialize + PartialEq> Response<T> {
    pub fn into_api_success(self) -> ApiSuccess<T> {
        match self {
            Response::OK(data) => ApiSuccess::new(StatusCode::OK, data),
            Response::Created(data) => ApiSuccess::new(StatusCode::CREATED, data),
            Response::Accepted(data) => ApiSuccess::new(StatusCode::ACCEPTED, data),
        }
    }
}
