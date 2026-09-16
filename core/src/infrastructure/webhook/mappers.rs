use std::collections::HashMap;

use chrono::{TimeZone, Utc};
use serde_json::from_value;

use ferriskey_domain::realm::RealmId;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::webhook::entities::retry_policy::RetryPolicyOverride;
use crate::domain::webhook::entities::webhook_delivery::{
    DeliveryErrorCode, DeliveryStatus, WebhookDelivery, WebhookDeliveryId,
};
use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use crate::domain::webhook::entities::{webhook::Webhook, webhook_subscriber::WebhookSubscriber};
use crate::entity::webhook_deliveries::Model as WebhookDeliveryModel;
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
            retry_policy: RetryPolicyOverride {
                max_attempts: value.retry_max_attempts.and_then(|v| u32::try_from(v).ok()),
                base_delay_ms: value
                    .retry_base_delay_ms
                    .and_then(|v| u32::try_from(v).ok()),
                max_delay_ms: value.retry_max_delay_ms.and_then(|v| u32::try_from(v).ok()),
                max_total_delay_ms: value
                    .retry_max_total_delay_ms
                    .and_then(|v| u32::try_from(v).ok()),
            },
            effective_retry_policy: None,
            description: value.description.clone(),
            name: value.name.clone(),
            headers,
            secret: value.secret.clone(),
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
            retry_policy: RetryPolicyOverride {
                max_attempts: value.retry_max_attempts.and_then(|v| u32::try_from(v).ok()),
                base_delay_ms: value
                    .retry_base_delay_ms
                    .and_then(|v| u32::try_from(v).ok()),
                max_delay_ms: value.retry_max_delay_ms.and_then(|v| u32::try_from(v).ok()),
                max_total_delay_ms: value
                    .retry_max_total_delay_ms
                    .and_then(|v| u32::try_from(v).ok()),
            },
            effective_retry_policy: None,
            description: value.description,
            name: value.name,
            headers,
            secret: value.secret,
            triggered_at,
            created_at,
            updated_at,
        }
    }
}

impl TryFrom<WebhookDeliveryModel> for WebhookDelivery {
    type Error = CoreError;

    fn try_from(value: WebhookDeliveryModel) -> Result<Self, Self::Error> {
        let event: WebhookTrigger = value
            .event
            .try_into()
            .map_err(|_| CoreError::InternalServerError)?;

        let status = DeliveryStatus::parse(&value.status).ok_or(CoreError::InternalServerError)?;

        let last_error_code = match value.last_error_code.as_deref() {
            Some(raw) => Some(DeliveryErrorCode::parse(raw).ok_or(CoreError::InternalServerError)?),
            None => None,
        };

        let last_status_code = match value.last_status_code {
            Some(code) => Some(u16::try_from(code).map_err(|_| CoreError::InternalServerError)?),
            None => None,
        };

        let attempt_count =
            u32::try_from(value.attempt_count).map_err(|_| CoreError::InternalServerError)?;

        Ok(Self {
            id: WebhookDeliveryId::from_uuid(value.id),
            realm_id: RealmId::from(value.realm_id),
            webhook_id: value.webhook_id,
            event,
            resource_id: value.resource_id,
            payload: value.payload,
            status,
            attempt_count,
            next_attempt_at: value.next_attempt_at.map(|at| Utc.from_utc_datetime(&at)),
            leased_until: value.leased_until.map(|at| Utc.from_utc_datetime(&at)),
            last_attempt_at: value.last_attempt_at.map(|at| Utc.from_utc_datetime(&at)),
            last_status_code,
            last_error_code,
            last_error_detail: value.last_error_detail,
            created_at: Utc.from_utc_datetime(&value.created_at),
            updated_at: Utc.from_utc_datetime(&value.updated_at),
        })
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
