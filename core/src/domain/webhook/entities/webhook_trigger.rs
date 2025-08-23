use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum WebhookTrigger {
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserBulkDeleted,
    UserAssignRole,
    UserUnassignRole,
    UserDeleteCredentials,
    AuthResetPassword,
}

impl ToString for WebhookTrigger {
    fn to_string(&self) -> String {
        match self {
            WebhookTrigger::UserCreated => "user.created".to_string(),
            WebhookTrigger::UserUpdated => "user.updated".to_string(),
            WebhookTrigger::UserDeleted => "user.deleted".to_string(),
            WebhookTrigger::UserBulkDeleted => "user.bulk_deleted".to_string(),
            WebhookTrigger::UserAssignRole => "user.assign.role".to_string(),
            WebhookTrigger::UserUnassignRole => "user.unassign.role".to_string(),
            WebhookTrigger::UserDeleteCredentials => "user.credentials_deleted".to_string(),
            WebhookTrigger::AuthResetPassword => "auth.reset_password".to_string(),
        }
    }
}

impl TryFrom<String> for WebhookTrigger {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "user.created" => Ok(WebhookTrigger::UserCreated),
            "user.updated" => Ok(WebhookTrigger::UserUpdated),
            "user.deleted" => Ok(WebhookTrigger::UserDeleted),
            "user.bulk_deleted" => Ok(WebhookTrigger::UserBulkDeleted),
            "user.assign.role" => Ok(WebhookTrigger::UserAssignRole),
            "user.unassign.role" => Ok(WebhookTrigger::UserUnassignRole),
            "user.credentials_deleted" => Ok(WebhookTrigger::UserDeleteCredentials),
            "auth.reset_password" => Ok(WebhookTrigger::AuthResetPassword),
            _ => Err("Invalid webhook trigger".to_string()),
        }
    }
}
