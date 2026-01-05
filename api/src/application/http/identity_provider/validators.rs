use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateIdentityProviderValidator {
    #[validate(length(min = 1, message = "alias is required"))]
    #[serde(default)]
    pub alias: String,

    #[validate(length(min = 1, message = "provider_id is required"))]
    #[serde(default)]
    pub provider_id: String,

    #[serde(default)]
    pub enabled: bool,

    #[serde(default)]
    pub display_name: Option<String>,

    #[serde(default)]
    pub first_broker_login_flow_alias: Option<String>,

    #[serde(default)]
    pub post_broker_login_flow_alias: Option<String>,

    #[serde(default)]
    pub store_token: bool,

    #[serde(default)]
    pub add_read_token_role_on_create: bool,

    #[serde(default)]
    pub trust_email: bool,

    #[serde(default)]
    pub link_only: bool,

    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateIdentityProviderValidator {
    #[serde(default)]
    pub enabled: Option<bool>,

    #[serde(default)]
    pub display_name: Option<String>,

    #[serde(default)]
    pub first_broker_login_flow_alias: Option<String>,

    #[serde(default)]
    pub post_broker_login_flow_alias: Option<String>,

    #[serde(default)]
    pub store_token: Option<bool>,

    #[serde(default)]
    pub add_read_token_role_on_create: Option<bool>,

    #[serde(default)]
    pub trust_email: Option<bool>,

    #[serde(default)]
    pub link_only: Option<bool>,

    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct IdentityProviderResponse {
    pub alias: String,
    pub internal_id: Uuid,
    pub provider_id: String,
    pub enabled: bool,
    pub display_name: Option<String>,
    pub first_broker_login_flow_alias: Option<String>,
    pub post_broker_login_flow_alias: Option<String>,
    pub store_token: bool,
    pub add_read_token_role_on_create: bool,
    pub trust_email: bool,
    pub link_only: bool,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct IdentityProvidersResponse {
    pub data: Vec<IdentityProviderResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeleteIdentityProviderResponse {
    pub count: u32,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListIdentityProvidersQuery {
    #[serde(default)]
    pub brief_representation: Option<bool>,
}
