use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{generate_random_string, generate_timestamp, realm::RealmId};

pub mod redirect_uri;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ClientType {
    Confidential,
    Public,
    System,
}

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceSessionStrategy {
    Terminate,
    #[default]
    Expire,
}

impl fmt::Display for MaintenanceSessionStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaintenanceSessionStrategy::Terminate => write!(f, "terminate"),
            MaintenanceSessionStrategy::Expire => write!(f, "expire"),
        }
    }
}

impl FromStr for MaintenanceSessionStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "terminate" => Ok(MaintenanceSessionStrategy::Terminate),
            "expire" => Ok(MaintenanceSessionStrategy::Expire),
            _ => Err(format!("unknown maintenance session strategy: {s}")),
        }
    }
}

impl fmt::Display for ClientType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientType::Confidential => write!(f, "confidential"),
            ClientType::Public => write!(f, "public"),
            ClientType::System => write!(f, "system"),
        }
    }
}

impl FromStr for ClientType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "confidential" => Ok(ClientType::Confidential),
            "public" => Ok(ClientType::Public),
            "system" => Ok(ClientType::System),
            _ => Err(format!("unknown client type: {s}")),
        }
    }
}

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
    pub client_type: ClientType,
    pub name: String,
    pub redirect_uris: Option<Vec<redirect_uri::RedirectUri>>,
    pub access_token_lifetime: Option<i64>,
    pub refresh_token_lifetime: Option<i64>,
    pub id_token_lifetime: Option<i64>,
    pub temporary_token_lifetime: Option<i64>,
    pub maintenance_enabled: bool,
    pub maintenance_reason: Option<String>,
    pub maintenance_session_strategy: MaintenanceSessionStrategy,
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
    pub client_type: ClientType,
    pub direct_access_grants_enabled: Option<bool>,
    pub access_token_lifetime: Option<i64>,
    pub refresh_token_lifetime: Option<i64>,
    pub id_token_lifetime: Option<i64>,
    pub temporary_token_lifetime: Option<i64>,
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
            access_token_lifetime: config.access_token_lifetime,
            refresh_token_lifetime: config.refresh_token_lifetime,
            id_token_lifetime: config.id_token_lifetime,
            temporary_token_lifetime: config.temporary_token_lifetime,
            maintenance_enabled: false,
            maintenance_reason: None,
            maintenance_session_strategy: MaintenanceSessionStrategy::default(),
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
            client_type: ClientType::Confidential,
            name: format!("{client_id} Client"),
            redirect_uris: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
            maintenance_enabled: false,
            maintenance_reason: None,
            maintenance_session_strategy: MaintenanceSessionStrategy::default(),
            created_at: now,
            updated_at: now,
        }
    }
}
