use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum WebhookTrigger {
    UserCreated,
}

impl ToString for WebhookTrigger {
    fn to_string(&self) -> String {
        match self {
            WebhookTrigger::UserCreated => "user.created".to_string(),
        }
    }
}

impl TryFrom<String> for WebhookTrigger {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "user.created" => Ok(WebhookTrigger::UserCreated),
            _ => Err("Invalid webhook trigger".to_string()),
        }
    }
}
