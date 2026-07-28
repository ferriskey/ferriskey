use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateEmailTemplateValidator {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,

    #[validate(length(min = 1, message = "email_type is required"))]
    pub email_type: String,

    pub structure: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateEmailTemplateValidator {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,

    pub structure: serde_json::Value,
}
