use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct WebhookSubscriber {
    pub id: Uuid,
    pub name: String,
    pub webhook_id: Uuid,
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
