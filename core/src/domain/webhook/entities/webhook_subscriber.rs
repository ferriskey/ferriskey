use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;

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
