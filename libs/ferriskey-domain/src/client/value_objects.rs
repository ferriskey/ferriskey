use serde::{Deserialize, Serialize};

use crate::authentication::entities::AuthProtocol;
use crate::client::entities::auth_method::{
    ClientAuthSettings, ClientJwks, TokenEndpointAuthMethod,
};
use crate::client::entities::{ClientRegistrationSource, ClientType, MaintenanceSessionStrategy};
use crate::realm::RealmId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientRequest {
    pub realm_id: RealmId,
    pub name: String,
    pub client_id: String,
    pub secret: Option<String>,
    pub enabled: bool,
    pub protocol: AuthProtocol,
    pub public_client: bool,
    pub service_account_enabled: bool,
    pub direct_access_grants_enabled: bool,
    pub oauth_device_code_grant_enabled: bool,
    pub token_exchange_enabled: bool,
    pub client_type: ClientType,
    /// Public clients have no secret to authenticate with, so PKCE is the only
    /// thing binding an authorization code to the instance that requested it.
    pub require_pkce: bool,
    pub consent_required: bool,
    pub registration_source: ClientRegistrationSource,
    pub auth: ClientAuthSettings,
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
            token_exchange_enabled: false,
            enabled: true,
            name: client_name,
            protocol: AuthProtocol::OpenIdConnect,
            public_client: true,
            secret: None,
            service_account_enabled: false,
            require_pkce: false,
            consent_required: false,
            registration_source: ClientRegistrationSource::Admin,
            auth: ClientAuthSettings::default_for(true),
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
    pub token_exchange_enabled: Option<bool>,
    pub require_pkce: Option<bool>,
    pub backchannel_logout_uri: Option<Option<String>>,
    pub backchannel_logout_session_required: Option<bool>,
    pub access_token_lifetime: Option<i64>,
    pub refresh_token_lifetime: Option<i64>,
    pub id_token_lifetime: Option<i64>,
    pub temporary_token_lifetime: Option<i64>,
    pub maintenance_enabled: Option<bool>,
    pub maintenance_reason: Option<Option<String>>,
    pub maintenance_session_strategy: Option<MaintenanceSessionStrategy>,
    pub consent_required: Option<bool>,
    pub token_endpoint_auth_method: Option<TokenEndpointAuthMethod>,
    pub jwks: Option<Option<ClientJwks>>,
    pub jwks_uri: Option<Option<String>>,
}

impl UpdateClientRequest {
    pub fn touches_authentication(&self) -> bool {
        self.token_endpoint_auth_method.is_some() || self.jwks.is_some() || self.jwks_uri.is_some()
    }

    /// The authentication settings `client` would have once this update applies.
    pub fn merged_auth(&self, client: &crate::client::entities::Client) -> ClientAuthSettings {
        ClientAuthSettings {
            method: self
                .token_endpoint_auth_method
                .unwrap_or(client.token_endpoint_auth_method),
            jwks: self.jwks.clone().unwrap_or_else(|| client.jwks.clone()),
            jwks_uri: self
                .jwks_uri
                .clone()
                .unwrap_or_else(|| client.jwks_uri.clone()),
        }
    }
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
pub struct CreateTokenExchangePolicyRequest {
    pub target_audience: String,
    pub allowed_scopes: Option<Vec<String>>,
    pub allow_impersonation: bool,
    pub allow_delegation: bool,
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
