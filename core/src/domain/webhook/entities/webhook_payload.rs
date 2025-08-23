use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;

pub struct WebhookPayload<T>
where
    T: Serialize + Send + Sync,
{
    pub event: WebhookTrigger,
    pub timestamp: String,
    pub resource_id: Uuid,
    pub data: Option<T>,
}

impl<T> WebhookPayload<T>
where
    T: Serialize + Send + Sync,
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
