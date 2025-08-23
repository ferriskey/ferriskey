use ferriskey_core::domain::webhook::entities::webhook_trigger::WebhookTrigger;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateWebhookValidator {
    #[validate(length(min = 1, message = "endpoint is required"))]
    #[serde(default)]
    pub endpoint: String,

    #[validate(length(min = 1, message = "subscribers is required"))]
    #[serde(default)]
    pub subscribers: Vec<WebhookTrigger>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateWebhookValidator {
    #[validate(length(min = 1, message = "endpoint is required"))]
    #[serde(default)]
    pub endpoint: String,

    #[validate(length(min = 1, message = "subscribers is required"))]
    #[serde(default)]
    pub subscribers: Vec<WebhookTrigger>,
}
