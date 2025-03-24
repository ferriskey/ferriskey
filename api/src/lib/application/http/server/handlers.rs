use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

pub struct ApiSuccess<T: Serialize + PartialEq>(StatusCode, Json<ApiResponseBody<T>>);

impl<T> PartialEq for ApiSuccess<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1.0 == other.1.0
    }
}

impl<T: Serialize + PartialEq> ApiSuccess<T> {
    pub fn new(status: StatusCode, data: T) -> Self {
        ApiSuccess(status, Json(ApiResponseBody::new(status, data)))
    }
}

impl<T: Serialize + PartialEq> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> axum::response::Response {
        (self.0, self.1).into_response()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiResponseBody<T: Serialize + PartialEq> {
    status_code: u16,
    data: T,
}

impl<T: Serialize + PartialEq> ApiResponseBody<T> {
    pub fn new(status_code: StatusCode, data: T) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data,
        }
    }
}

impl ApiResponseBody<ApiErrorData> {
    pub fn new_error(status_code: StatusCode, message: String) -> Self {
        Self {
            status_code: status_code.as_u16(),
            data: ApiErrorData { message },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiErrorData {
    pub message: String,
}

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
