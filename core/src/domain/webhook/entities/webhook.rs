use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{
    common::generate_timestamp, webhook::entities::webhook_subscriber::WebhookSubscriber,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct Webhook {
    pub id: Uuid,
    pub endpoint: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub subscribers: Vec<WebhookSubscriber>,
    pub triggered_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Webhook {
    pub fn new(
        endpoint: String,
        subscribers: Vec<WebhookSubscriber>,
        name: Option<String>,
        description: Option<String>,
        triggered_at: Option<DateTime<Utc>>,
        updated_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        let (_, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            endpoint,
            name,
            description,
            subscribers,
            triggered_at,
            updated_at,
            created_at,
        }
    }
}
