use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct Webhook {
    pub id: Uuid,
    pub endpoint: String,
    pub subscribers: Vec<WebhookSubscriber>,
    pub triggered_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct WebhookSubscriber {
    pub id: Uuid,
    pub name: String,
    pub webhook_id: Uuid,
}

#[derive(Debug, Clone, Error)]
pub enum WebhookError {
    #[error("Webhook not found")]
    NotFound,

    #[error("Internal server error")]
    InternalServerError,

    #[error("Forbidden")]
    Forbidden,

    #[error("Realm not found")]
    RealmNotFound,
}

impl Webhook {
    pub fn new(
        endpoint: String,
        subscribers: Vec<WebhookSubscriber>,
        triggered_at: Option<DateTime<Utc>>,
        updated_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        let (_, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            endpoint,
            subscribers,
            triggered_at,
            updated_at,
            created_at,
        }
    }
}

impl WebhookSubscriber {
    pub fn new(id: Uuid, name: String, webhook_id: Uuid) -> Self {
        Self {
            id,
            name,
            webhook_id,
        }
    }
}
