use std::collections::HashMap;

use uuid::Uuid;

use crate::entities::trigger::WebhookTrigger;

pub struct GetWebhooksInput {
    pub realm_name: String,
}

pub struct GetWebhookInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
}

pub struct GetWebhookSubscribersInput {
    pub realm_name: String,
    pub subscriber: WebhookTrigger,
}

pub struct CreateWebhookInput {
    pub realm_name: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: String,
    pub headers: HashMap<String, String>,
    pub subscribers: Vec<WebhookTrigger>,
}

pub struct UpdateWebhookInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: String,
    pub headers: HashMap<String, String>,
    pub subscribers: Vec<WebhookTrigger>,
}

pub struct DeleteWebhookInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
}
