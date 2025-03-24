use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordValidator {
    #[serde(default)]
    pub temporary: bool,
    #[serde(default)]
    pub credential_type: String,
    #[validate(length(min = 1, message = "value is required"))]
    #[serde(default)]
    pub value: String,
}
