use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::{NoContext, Timestamp, Uuid};

use crate::domain::client::{
    entities::redirect_uri::RedirectUri,
    value_objects::{CreateRedirectUriRequest, UpdateClientRequest},
};

pub mod redirect_uri;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, ToSchema)]
pub struct Client {
    pub id: Uuid,
    pub enabled: bool,
    pub client_id: String,
    pub secret: Option<String>,
    pub realm_id: Uuid,
    pub protocol: String,
    pub public_client: bool,
    pub service_account_enabled: bool,
    pub direct_access_grants_enabled: bool,
    pub client_type: String,
    pub name: String,
    pub redirect_uris: Option<Vec<RedirectUri>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ClientConfig {
    pub realm_id: Uuid,
    pub name: String,
    pub client_id: String,
    pub secret: Option<String>,
    pub enabled: bool,
    pub protocol: String,
    pub public_client: bool,
    pub service_account_enabled: bool,
    pub client_type: String,
    pub direct_access_grants_enabled: Option<bool>,
}

impl Client {
    pub fn new(config: ClientConfig) -> Self {
        let now = Utc::now();
        let seconds = now.timestamp().try_into().unwrap_or(0);

        let timestamp = Timestamp::from_unix(NoContext, seconds, 0);
        Self {
            id: Uuid::new_v7(timestamp),
            enabled: config.enabled,
            client_id: config.client_id,
            secret: config.secret,
            realm_id: config.realm_id,
            protocol: config.protocol,
            public_client: config.public_client,
            service_account_enabled: config.service_account_enabled,
            direct_access_grants_enabled: config.direct_access_grants_enabled.unwrap_or_default(),
            client_type: config.client_type,
            name: config.name,
            redirect_uris: None,
            created_at: now,
            updated_at: now,
        }
    }
}

pub struct CreateClientInput {
    pub realm_name: String,
    pub name: String,
    pub client_id: String,
    pub client_type: String,
    pub service_account_enabled: bool,
    pub public_client: bool,
    pub protocol: String,
    pub enabled: bool,
    pub direct_access_grants_enabled: bool,
}

pub struct CreateRedirectUriInput {
    pub client_id: Uuid,
    pub realm_name: String,
    pub payload: CreateRedirectUriRequest,
}

pub struct CreateRoleInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub description: Option<String>,
    pub name: String,
    pub permissions: Vec<String>,
}

pub struct DeleteClientInput {
    pub realm_name: String,
    pub client_id: Uuid,
}

pub struct DeleteRedirectUriInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub uri_id: Uuid,
}

pub struct GetClientInput {
    pub client_id: Uuid,
    pub realm_name: String,
}

pub struct GetClientRolesInput {
    pub client_id: Uuid,
    pub realm_name: String,
}

pub struct GetRedirectUrisInput {
    pub realm_name: String,
    pub client_id: Uuid,
}

pub struct GetClientsInput {
    pub realm_name: String,
}

pub struct UpdateClientInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub payload: UpdateClientRequest,
}

pub struct UpdateRedirectUriInput {
    pub realm_name: String,
    pub client_id: Uuid,
    pub redirect_uri_id: Uuid,
    pub enabled: bool,
}
