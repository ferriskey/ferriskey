use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::{NoContext, Timestamp, Uuid};

use crate::domain::client::entities::redirect_uri::RedirectUri;

pub mod redirect_uri;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, ToSchema)]
#[typeshare]
pub struct Client {
    #[typeshare(serialized_as = "string")]
    pub id: Uuid,
    pub enabled: bool,
    pub client_id: String,
    pub secret: Option<String>,
    #[typeshare(serialized_as = "string")]
    pub realm_id: Uuid,
    pub protocol: String,
    pub public_client: bool,
    pub service_account_enabled: bool,
    pub client_type: String,
    pub name: String,
    pub redirect_uris: Option<Vec<RedirectUri>>,
    #[typeshare(serialized_as = "Date")]
    pub created_at: DateTime<Utc>,
    #[typeshare(serialized_as = "Date")]
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
}

#[derive(Debug, Clone, Error)]
pub enum ClientError {
    #[error("Client not found")]
    NotFound,
    #[error("Client already exists")]
    AlreadyExists,
    #[error("Invalid client")]
    Invalid,
    #[error("Internal server error")]
    InternalServerError,
    #[error("Redirect URI not found")]
    RedirectUriNotFound,
    #[error("Invalid redirect URI")]
    InvalidRedirectUri,
    #[error("{0}")]
    Forbidden(String),
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
            client_type: config.client_type,
            name: config.name,
            redirect_uris: None,
            created_at: now,
            updated_at: now,
        }
    }
}
