use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateClientValidator {
    #[validate(length(min = 1, message = "name is required"))]
    #[serde(default)]
    pub name: String,
    #[validate(length(min = 1, message = "client_id is required"))]
    #[serde(default)]
    pub client_id: String,
    #[validate(length(min = 1, message = "client_type is required"))]
    #[serde(default)]
    pub client_type: String,
    #[serde(default)]
    pub service_account_enabled: bool,
    #[serde(default)]
    pub public_client: bool,
    #[validate(length(min = 1, message = "protocol is required"))]
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateRedirectUriValidator {
    #[validate(length(min = 1, message = "Uri value is required"))]
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub enabled: bool,
}
