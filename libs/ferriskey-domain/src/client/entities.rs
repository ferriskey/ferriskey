use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{generate_random_string, generate_timestamp, realm::RealmId};

pub mod redirect_uri;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, ToSchema)]
pub struct Client {
    pub id: Uuid,
    pub enabled: bool,
    pub client_id: String,
    pub secret: Option<String>,
    pub realm_id: RealmId,
    pub protocol: String,
    pub public_client: bool,
    pub service_account_enabled: bool,
    pub direct_access_grants_enabled: bool,
    pub client_type: String,
    pub name: String,
    pub redirect_uris: Option<Vec<redirect_uri::RedirectUri>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ClientConfig {
    pub realm_id: RealmId,
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
        let (now, timestamp) = generate_timestamp();
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

    pub fn from_realm_and_client_id(realm_id: RealmId, client_id: String) -> Self {
        let (now, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            enabled: true,
            client_id: client_id.clone(),
            secret: Some(generate_random_string()),
            realm_id,
            protocol: "openid-connect".to_string(),
            public_client: false,
            service_account_enabled: false,
            direct_access_grants_enabled: false,
            client_type: "confidential".to_string(),
            name: format!("{client_id} Client"),
            redirect_uris: None,
            created_at: now,
            updated_at: now,
        }
    }
}
