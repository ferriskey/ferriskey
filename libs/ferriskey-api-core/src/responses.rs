use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResponseBody<T: Serialize> {
    status_code: u16,
    data: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorResponseData {
    pub message: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Serialize)]
pub struct ApiErrorResponse {
    pub errors: Vec<ApiErrorDetail>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Serialize)]
pub struct ApiErrorDetail {
    pub message: String,
    pub rule: String,
    pub field: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Serialize)]
pub struct ApiResponseError {
    pub code: String,
    pub status: u16,
    pub message: String,
}
