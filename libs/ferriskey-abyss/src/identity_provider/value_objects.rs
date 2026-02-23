use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use ferriskey_domain::realm::RealmId;

/// Request DTO for creating a new identity provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIdentityProviderRequest {
    pub realm_id: RealmId,
    pub alias: String,
    pub provider_id: String,
    pub enabled: bool,
    pub display_name: Option<String>,
    pub first_broker_login_flow_alias: Option<String>,
    pub post_broker_login_flow_alias: Option<String>,
    pub store_token: bool,
    pub add_read_token_role_on_create: bool,
    pub trust_email: bool,
    pub link_only: bool,
    pub config: JsonValue,
}

/// Request DTO for updating an existing identity provider
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateIdentityProviderRequest {
    pub enabled: Option<bool>,
    pub display_name: Option<String>,
    pub first_broker_login_flow_alias: Option<String>,
    pub post_broker_login_flow_alias: Option<String>,
    pub store_token: Option<bool>,
    pub add_read_token_role_on_create: Option<bool>,
    pub trust_email: Option<bool>,
    pub link_only: Option<bool>,
    pub config: Option<JsonValue>,
}
