use std::fmt::{Debug, Display};

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
    ClientCreated,
    ClientUpdated,
    ClientDeleted,
    ClientRoleCreated,
    ClientRoleUpdated,
    RedirectUriCreated,
    RedirectUriUpdated,
    RoleCreated,
    RoleUpdated,
    RealmCreated,
    RealmUpdated,
    RealmDeleted,
    RealmSettingsUpdated,
    WebhookCreated,
    WebhookUpdated,
    WebhookDeleted,
}

impl Display for WebhookTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookTrigger::UserCreated => write!(f, "user.created"),
            WebhookTrigger::UserUpdated => write!(f, "user.updated"),
            WebhookTrigger::UserDeleted => write!(f, "user.deleted"),
            WebhookTrigger::UserBulkDeleted => write!(f, "user.bulk_deleted"),
            WebhookTrigger::UserAssignRole => write!(f, "user.assign.role"),
            WebhookTrigger::UserUnassignRole => write!(f, "user.unassign.role"),
            WebhookTrigger::UserDeleteCredentials => write!(f, "user.credentials_deleted"),
            WebhookTrigger::AuthResetPassword => write!(f, "auth.reset_password"),
            WebhookTrigger::ClientCreated => write!(f, "client.created"),
            WebhookTrigger::ClientUpdated => write!(f, "client.updated"),
            WebhookTrigger::ClientDeleted => write!(f, "client.deleted"),
            WebhookTrigger::ClientRoleCreated => write!(f, "client.role_created"),
            WebhookTrigger::ClientRoleUpdated => write!(f, "client.role_updated"),
            WebhookTrigger::RedirectUriCreated => write!(f, "redirect_uri.created"),
            WebhookTrigger::RedirectUriUpdated => write!(f, "redirect_uri.updated"),
            WebhookTrigger::RoleCreated => write!(f, "role.created"),
            WebhookTrigger::RoleUpdated => write!(f, "role.updated"),
            WebhookTrigger::RealmCreated => write!(f, "realm.created"),
            WebhookTrigger::RealmUpdated => write!(f, "realm.updated"),
            WebhookTrigger::RealmDeleted => write!(f, "realm.deleted"),
            WebhookTrigger::RealmSettingsUpdated => write!(f, "realm.settings.updated"),
            WebhookTrigger::WebhookCreated => write!(f, "webhook.created"),
            WebhookTrigger::WebhookUpdated => write!(f, "webhook.updated"),
            WebhookTrigger::WebhookDeleted => write!(f, "webhook.deleted"),
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
            "client.created" => Ok(WebhookTrigger::ClientCreated),
            "client.updated" => Ok(WebhookTrigger::ClientUpdated),
            "client.deleted" => Ok(WebhookTrigger::ClientDeleted),
            "client.role_created" => Ok(WebhookTrigger::ClientRoleCreated),
            "client.role_updated" => Ok(WebhookTrigger::ClientRoleUpdated),
            "redirect_uri.created" => Ok(WebhookTrigger::RedirectUriCreated),
            "redirect_uri.updated" => Ok(WebhookTrigger::RedirectUriUpdated),
            "role.created" => Ok(WebhookTrigger::RoleCreated),
            "role.updated" => Ok(WebhookTrigger::RoleUpdated),
            "realm.created" => Ok(WebhookTrigger::RealmCreated),
            "realm.updated" => Ok(WebhookTrigger::RealmUpdated),
            "realm.deleted" => Ok(WebhookTrigger::RealmDeleted),
            "realm.settings.updated" => Ok(WebhookTrigger::RealmSettingsUpdated),
            "webhook.created" => Ok(WebhookTrigger::WebhookCreated),
            "webhook.updated" => Ok(WebhookTrigger::WebhookUpdated),
            "webhook.deleted" => Ok(WebhookTrigger::WebhookDeleted),
            _ => Err("Invalid webhook trigger".to_string()),
        }
    }
}
