use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::{NoContext, Timestamp, Uuid};

use crate::entities::trigger::WebhookTrigger;

pub(crate) mod inputs;
pub(crate) mod trigger;

pub struct Webhook {
    pub id: Uuid,
    pub endpoint: String,
    pub headers: HashMap<String, String>,
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
        let now = Utc::now();
        let seconds = now.timestamp().try_into().unwrap_or(0);
        let timestamp = Timestamp::from_unix(NoContext, seconds, 0);
        Self {
            id: Uuid::new_v7(timestamp),
            headers: HashMap::new(),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct WebhookSubscriber {
    pub id: Uuid,
    pub name: WebhookTrigger,
    pub webhook_id: Uuid,
}

impl WebhookSubscriber {
    pub fn new(id: Uuid, name: WebhookTrigger, webhook_id: Uuid) -> Self {
        Self {
            id,
            name,
            webhook_id,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WebhookPayload<T>
where
    T: Serialize + Send + Sync + Clone + 'static,
{
    pub event: WebhookTrigger,
    pub timestamp: String,
    pub resource_id: Uuid,
    pub data: Option<T>,
}

impl<T> WebhookPayload<T>
where
    T: Serialize + Send + Sync + Clone + 'static,
{
    pub fn new(event: WebhookTrigger, resource_id: Uuid, data: Option<T>) -> Self {
        WebhookPayload {
            event,
            timestamp: Utc::now().to_rfc3339(),
            resource_id,
            data,
        }
    }
}
