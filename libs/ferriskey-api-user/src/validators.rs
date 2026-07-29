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

    #[serde(default)]
    pub firstname: Option<String>,

    #[serde(default)]
    pub lastname: Option<String>,

    #[serde(default)]
    pub email: Option<String>,

    #[serde(default)]
    pub email_verified: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct BulkDeleteUserValidator {
    #[serde(default)]
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateUserValidator {
    #[serde(default)]
    pub firstname: Option<String>,

    #[serde(default)]
    pub lastname: Option<String>,

    #[serde(default)]
    pub email: Option<String>,

    #[serde(default)]
    pub email_verified: Option<bool>,

    #[serde(default)]
    pub enabled: Option<bool>,

    #[serde(default)]
    pub required_actions: Option<Vec<String>>,
}
