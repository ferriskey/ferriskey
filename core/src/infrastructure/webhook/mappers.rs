use chrono::{TimeZone, Utc};

use crate::domain::webhook::entities::{Webhook, WebhookSubscriber};
use crate::entity::webhook_subscribers::Model as WebhookSubscriberModel;
use crate::entity::webhooks::Model as WebhookModel;

impl From<&WebhookModel> for Webhook {
    fn from(value: &WebhookModel) -> Self {
        let triggered_at = if let Some(triggered_at) = value.triggered_at {
            Some(Utc.from_utc_datetime(&triggered_at))
        } else {
            None
        };
        let created_at = Utc.from_utc_datetime(&value.created_at);
        let updated_at = Utc.from_utc_datetime(&value.updated_at);

        Self {
            id: value.id,
            endpoint: value.endpoint.clone(),
            subscribers: Vec::new(),
            triggered_at,
            created_at,
            updated_at,
        }
    }
}

impl From<WebhookModel> for Webhook {
    fn from(value: WebhookModel) -> Self {
        let triggered_at = if let Some(triggered_at) = value.triggered_at {
            Some(Utc.from_utc_datetime(&triggered_at))
        } else {
            None
        };
        let created_at = Utc.from_utc_datetime(&value.created_at);
        let updated_at = Utc.from_utc_datetime(&value.updated_at);

        Self {
            id: value.id,
            endpoint: value.endpoint.clone(),
            subscribers: Vec::new(),
            triggered_at,
            created_at,
            updated_at,
        }
    }
}

impl From<WebhookSubscriberModel> for WebhookSubscriber {
    fn from(value: WebhookSubscriberModel) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            webhook_id: value.webhook_id,
        }
    }
}
