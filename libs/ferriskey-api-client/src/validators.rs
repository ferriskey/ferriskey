use ferriskey_core::domain::client::entities::ClientType;
use ferriskey_core::domain::client::entities::saml::{NameIdFormat, SamlAttributeNameFormat};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct EvaluateScopesValidator {
    /// User whose token would be previewed.
    pub user_id: Uuid,
    /// Requested scope string (space-separated). Default scopes always apply; optional scopes
    /// apply only when named here.
    #[serde(default)]
    pub scope: Option<String>,
}

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
    #[serde(default)]
    pub oauth_device_code_grant_enabled: bool,
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
    pub oauth_device_code_grant_enabled: Option<bool>,

    #[serde(default)]
    pub require_pkce: Option<bool>,

    #[serde(default)]
    pub access_token_lifetime: Option<i64>,

    #[serde(default)]
    pub refresh_token_lifetime: Option<i64>,

    #[serde(default)]
    pub id_token_lifetime: Option<i64>,

    #[serde(default)]
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

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateWebOriginValidator {
    #[validate(length(min = 1, message = "Origin value is required"))]
    #[serde(default)]
    pub value: String,
}

fn assertions_are_signed_by_default() -> bool {
    true
}

fn default_name_id_format() -> String {
    NameIdFormat::default().to_string()
}

fn default_attribute_name_format() -> String {
    SamlAttributeNameFormat::default().to_string()
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct SetClientSamlConfigValidator {
    #[validate(length(min = 1, message = "sp_entity_id is required"))]
    #[serde(default)]
    pub sp_entity_id: String,

    #[validate(length(min = 1, message = "acs_url is required"))]
    #[serde(default)]
    pub acs_url: String,

    #[validate(length(min = 1, message = "name_id_format is required"))]
    #[serde(default = "default_name_id_format")]
    pub name_id_format: String,

    #[serde(default = "assertions_are_signed_by_default")]
    pub sign_assertions: bool,

    #[serde(default)]
    pub sign_documents: bool,

    #[serde(default)]
    pub want_authn_requests_signed: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateSamlAttributeMapperValidator {
    #[validate(length(min = 1, message = "name is required"))]
    #[serde(default)]
    pub name: String,

    #[validate(length(min = 1, message = "name_format is required"))]
    #[serde(default = "default_attribute_name_format")]
    pub name_format: String,

    #[validate(length(min = 1, message = "source is required"))]
    #[serde(default)]
    pub source: String,
}
