use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct OtpVerifyRequest {
    pub code: String,
    pub label: String,
    pub secret: String,
}
