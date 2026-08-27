use serde::{Deserialize, Serialize};

use crate::client::entities::{ClientType, MaintenanceSessionStrategy};
use crate::realm::RealmId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientRequest {
    pub realm_id: RealmId,
    pub name: String,
    pub client_id: String,
    pub secret: Option<String>,
    pub enabled: bool,
    pub protocol: String,
    pub public_client: bool,
    pub service_account_enabled: bool,
    pub direct_access_grants_enabled: bool,
    pub oauth_device_code_grant_enabled: bool,
    pub client_type: ClientType,
    /// Public clients have no secret to authenticate with, so PKCE is the only
    /// thing binding an authorization code to the instance that requested it.
    pub require_pkce: bool,
}

impl CreateClientRequest {
    pub fn create_realm_system_client(
        realm_id: RealmId,
        client_name: String,
    ) -> CreateClientRequest {
        CreateClientRequest {
            realm_id,
            client_id: client_name.clone(),
            client_type: ClientType::System,
            direct_access_grants_enabled: false,
            oauth_device_code_grant_enabled: false,
            enabled: true,
            name: client_name,
            protocol: "openid-connect".to_string(),
            public_client: true,
            secret: None,
            service_account_enabled: false,
            require_pkce: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientRequest {
    pub name: Option<String>,
    pub client_id: Option<String>,
    pub enabled: Option<bool>,
    pub direct_access_grants_enabled: Option<bool>,
    pub oauth_device_code_grant_enabled: Option<bool>,
    pub require_pkce: Option<bool>,
    pub access_token_lifetime: Option<i64>,
    pub refresh_token_lifetime: Option<i64>,
    pub id_token_lifetime: Option<i64>,
    pub temporary_token_lifetime: Option<i64>,
    pub maintenance_enabled: Option<bool>,
    pub maintenance_reason: Option<Option<String>>,
    pub maintenance_session_strategy: Option<MaintenanceSessionStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRedirectUriRequest {
    pub value: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebOriginRequest {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetClientSamlConfigRequest {
    pub sp_entity_id: String,
    pub acs_url: String,
    pub name_id_format: String,
    pub sign_assertions: bool,
    pub sign_documents: bool,
    pub want_authn_requests_signed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSamlAttributeMapperRequest {
    pub name: String,
    pub name_format: String,
    pub source: String,
}
