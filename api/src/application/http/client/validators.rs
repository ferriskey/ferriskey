use ferriskey_core::domain::client::entities::ClientType;
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
    pub client_type: ClientType,
    #[serde(default)]
    pub service_account_enabled: bool,
    #[serde(default)]
    pub public_client: bool,
    #[validate(length(min = 1, message = "protocol is required"))]
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub direct_access_grants_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateClientValidator {
    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub client_id: Option<String>,

    #[serde(default)]
    pub enabled: Option<bool>,

    #[serde(default)]
    pub direct_access_grants_enabled: Option<bool>,

    #[serde(default)]
    #[validate(range(
        min = 60,
        max = 86400,
        message = "access_token_lifetime must be between 60 and 86400 seconds"
    ))]
    pub access_token_lifetime: Option<i64>,

    #[serde(default)]
    #[validate(range(
        min = 300,
        max = 2592000,
        message = "refresh_token_lifetime must be between 300 and 2592000 seconds"
    ))]
    pub refresh_token_lifetime: Option<i64>,

    #[serde(default)]
    #[validate(range(
        min = 60,
        max = 86400,
        message = "id_token_lifetime must be between 60 and 86400 seconds"
    ))]
    pub id_token_lifetime: Option<i64>,

    #[serde(default)]
    #[validate(range(
        min = 60,
        max = 86400,
        message = "temporary_token_lifetime must be between 60 and 86400 seconds"
    ))]
    pub temporary_token_lifetime: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateRedirectUriValidator {
    #[validate(length(min = 1, message = "Uri value is required"))]
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateRedirectUriValidator {
    #[serde(default)]
    pub enabled: bool,
}
