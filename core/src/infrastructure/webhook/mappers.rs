use std::collections::HashMap;

use chrono::{TimeZone, Utc};
use serde_json::from_value;

use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::entities::{webhook::Webhook, webhook_subscriber::WebhookSubscriber};
use crate::entity::webhook_subscribers::Model as WebhookSubscriberModel;
use crate::entity::webhooks::Model as WebhookModel;

impl From<&WebhookModel> for Webhook {
    fn from(value: &WebhookModel) -> Self {
        let created_at = Utc.from_utc_datetime(&value.created_at);
        let updated_at = Utc.from_utc_datetime(&value.updated_at);
        let triggered_at = value
            .triggered_at
            .map(|triggered_at| Utc.from_utc_datetime(&triggered_at));

        let headers =
            from_value::<HashMap<String, String>>(value.headers.clone()).unwrap_or_default();

        Self {
            id: value.id,
            endpoint: value.endpoint.clone(),
            subscribers: Vec::new(),
            description: value.description.clone(),
            name: value.name.clone(),
            headers,
            triggered_at,
            created_at,
            updated_at,
        }
    }
}

impl From<WebhookModel> for Webhook {
    fn from(value: WebhookModel) -> Self {
        let created_at = Utc.from_utc_datetime(&value.created_at);
        let updated_at = Utc.from_utc_datetime(&value.updated_at);
        let triggered_at = value
            .triggered_at
            .map(|triggered_at| Utc.from_utc_datetime(&triggered_at));

        let headers =
            from_value::<HashMap<String, String>>(value.headers.clone()).unwrap_or_default();

        Self {
            id: value.id,
            endpoint: value.endpoint.clone(),
            subscribers: Vec::new(),
            description: value.description,
            name: value.name,
            headers,
            triggered_at,
            created_at,
            updated_at,
        }
    }
}

impl TryFrom<WebhookSubscriberModel> for WebhookSubscriber {
    type Error = anyhow::Error;

    fn try_from(value: WebhookSubscriberModel) -> Result<Self, Self::Error> {
        let webhook_trigger: WebhookTrigger = value
            .name
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid webhook trigger"))?;

        Ok(Self {
            id: value.id,
            name: webhook_trigger,
            webhook_id: value.webhook_id,
        })
    }
}
