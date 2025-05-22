use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
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

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateUserValidator {
    #[validate(length(min = 1, message = "username is required"))]
    #[serde(default)]
    pub username: String,

    #[validate(length(min = 1, message = "firstname is required"))]
    #[serde(default)]
    pub firstname: String,

    #[validate(length(min = 1, message = "lastname is required"))]
    #[serde(default)]
    pub lastname: String,

    #[validate(length(min = 1, message = "email is required"))]
    #[serde(default)]
    pub email: String,

    #[serde(default)]
    pub email_verified: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct BulkDeleteUserValidator {
    #[serde(default)]
    pub ids: Vec<Uuid>,
}
