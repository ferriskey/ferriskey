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

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use uuid::Uuid;

    use super::*;
    use crate::entities::trigger::WebhookTrigger;

    #[test]
    fn webhook_new_sets_fields() {
        let endpoint = "https://example.com/webhooks".to_string();
        let webhook_id = Uuid::new_v4();
        let subscriber_id = Uuid::new_v4();
        let subscribers = vec![WebhookSubscriber::new(
            subscriber_id,
            WebhookTrigger::UserCreated,
            webhook_id,
        )];
        let name = Some("user hook".to_string());
        let description = Some("user created hook".to_string());
        let triggered_at = Some(Utc.with_ymd_and_hms(2024, 1, 2, 3, 4, 5).unwrap());
        let updated_at = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let created_at = Utc.with_ymd_and_hms(2023, 12, 31, 23, 59, 59).unwrap();

        let webhook = Webhook::new(
            endpoint.clone(),
            subscribers.clone(),
            name.clone(),
            description.clone(),
            triggered_at,
            updated_at,
            created_at,
        );

        assert_ne!(webhook.id, Uuid::nil());
        assert_eq!(webhook.endpoint, endpoint);
        assert!(webhook.headers.is_empty());
        assert_eq!(webhook.subscribers, subscribers);
        assert_eq!(webhook.name, name);
        assert_eq!(webhook.description, description);
        assert_eq!(webhook.triggered_at, triggered_at);
        assert_eq!(webhook.updated_at, updated_at);
        assert_eq!(webhook.created_at, created_at);
    }

    #[test]
    fn webhook_subscriber_new_sets_fields() {
        let id = Uuid::new_v4();
        let webhook_id = Uuid::new_v4();
        let subscriber = WebhookSubscriber::new(id, WebhookTrigger::RealmUpdated, webhook_id);

        assert_eq!(subscriber.id, id);
        assert_eq!(subscriber.name, WebhookTrigger::RealmUpdated);
        assert_eq!(subscriber.webhook_id, webhook_id);
    }

    #[test]
    fn webhook_payload_new_builds_timestamp_and_data() {
        let resource_id = Uuid::new_v4();
        let payload = WebhookPayload::new(
            WebhookTrigger::WebhookCreated,
            resource_id,
            Some("payload".to_string()),
        );

        assert_eq!(payload.event, WebhookTrigger::WebhookCreated);
        assert_eq!(payload.resource_id, resource_id);
        assert_eq!(payload.data, Some("payload".to_string()));
        assert!(chrono::DateTime::parse_from_rfc3339(&payload.timestamp).is_ok());
    }

    #[test]
    fn webhook_payload_new_allows_none_data() {
        let resource_id = Uuid::new_v4();
        let payload: WebhookPayload<String> =
            WebhookPayload::new(WebhookTrigger::WebhookDeleted, resource_id, None);

        assert_eq!(payload.event, WebhookTrigger::WebhookDeleted);
        assert_eq!(payload.resource_id, resource_id);
        assert!(payload.data.is_none());
    }
}
