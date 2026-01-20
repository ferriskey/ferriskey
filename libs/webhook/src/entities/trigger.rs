use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum WebhookTrigger {
    #[serde(rename = "user.created")]
    UserCreated,
    #[serde(rename = "user.updated")]
    UserUpdated,
    #[serde(rename = "user.deleted")]
    UserDeleted,
    #[serde(rename = "user.role.assigned")]
    UserRoleAssigned,
    #[serde(rename = "user.role.unassigned")]
    UserRoleUnassigned,
    #[serde(rename = "user.bulk_deleted")]
    UserBulkDeleted,
    #[serde(rename = "user.credentials.deleted")]
    UserDeleteCredentials,
    #[serde(rename = "auth.reset_password")]
    AuthResetPassword,
    #[serde(rename = "client.created")]
    ClientCreated,
    #[serde(rename = "client.updated")]
    ClientUpdated,
    #[serde(rename = "client.deleted")]
    ClientDeleted,
    #[serde(rename = "client.role.created")]
    ClientRoleCreated,
    #[serde(rename = "client.role.updated")]
    ClientRoleUpdated,
    #[serde(rename = "redirect_uri.created")]
    RedirectUriCreated,
    #[serde(rename = "redirect_uri.updated")]
    RedirectUriUpdated,
    #[serde(rename = "redirect_uri.deleted")]
    RedirectUriDeleted,
    #[serde(rename = "role.created")]
    RoleCreated,
    #[serde(rename = "role.updated")]
    RoleUpdated,
    #[serde(rename = "role.deleted")]
    RoleDeleted,
    #[serde(rename = "role.permission.updated")]
    RolePermissionUpdated,
    #[serde(rename = "realm.created")]
    RealmCreated,
    #[serde(rename = "realm.updated")]
    RealmUpdated,
    #[serde(rename = "realm.deleted")]
    RealmDeleted,
    #[serde(rename = "realm.settings.updated")]
    RealmSettingsUpdated,
    #[serde(rename = "webhook.created")]
    WebhookCreated,
    #[serde(rename = "webhook.updated")]
    WebhookUpdated,
    #[serde(rename = "webhook.deleted")]
    WebhookDeleted,
}

impl Display for WebhookTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookTrigger::UserCreated => write!(f, "user.created"),
            WebhookTrigger::UserUpdated => write!(f, "user.updated"),
            WebhookTrigger::UserDeleted => write!(f, "user.deleted"),
            WebhookTrigger::UserBulkDeleted => write!(f, "user.bulk_deleted"),
            WebhookTrigger::UserDeleteCredentials => write!(f, "user.credentials.deleted"),
            WebhookTrigger::UserRoleAssigned => write!(f, "user.role.assigned"),
            WebhookTrigger::UserRoleUnassigned => write!(f, "user.role.unassigned"),
            WebhookTrigger::AuthResetPassword => write!(f, "auth.reset_password"),
            WebhookTrigger::ClientCreated => write!(f, "client.created"),
            WebhookTrigger::ClientUpdated => write!(f, "client.updated"),
            WebhookTrigger::ClientDeleted => write!(f, "client.deleted"),
            WebhookTrigger::ClientRoleCreated => write!(f, "client.role.created"),
            WebhookTrigger::ClientRoleUpdated => write!(f, "client.role.updated"),
            WebhookTrigger::RedirectUriCreated => write!(f, "redirect_uri.created"),
            WebhookTrigger::RedirectUriUpdated => write!(f, "redirect_uri.updated"),
            WebhookTrigger::RedirectUriDeleted => write!(f, "redirect_uri.deleted"),
            WebhookTrigger::RoleCreated => write!(f, "role.created"),
            WebhookTrigger::RoleUpdated => write!(f, "role.updated"),
            WebhookTrigger::RolePermissionUpdated => write!(f, "role.permission.updated"),
            WebhookTrigger::RoleDeleted => write!(f, "role.deleted"),
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
            "user.credentials.deleted" => Ok(WebhookTrigger::UserDeleteCredentials),
            "user.role.assigned" => Ok(WebhookTrigger::UserRoleAssigned),
            "user.role.unassigned" => Ok(WebhookTrigger::UserRoleUnassigned),
            "auth.reset_password" => Ok(WebhookTrigger::AuthResetPassword),
            "client.created" => Ok(WebhookTrigger::ClientCreated),
            "client.updated" => Ok(WebhookTrigger::ClientUpdated),
            "client.deleted" => Ok(WebhookTrigger::ClientDeleted),
            "client.role.created" => Ok(WebhookTrigger::ClientRoleCreated),
            "client.role.updated" => Ok(WebhookTrigger::ClientRoleUpdated),
            "redirect_uri.created" => Ok(WebhookTrigger::RedirectUriCreated),
            "redirect_uri.updated" => Ok(WebhookTrigger::RedirectUriUpdated),
            "redirect_uri.deleted" => Ok(WebhookTrigger::RedirectUriDeleted),
            "role.created" => Ok(WebhookTrigger::RoleCreated),
            "role.updated" => Ok(WebhookTrigger::RoleUpdated),
            "role.permission.updated" => Ok(WebhookTrigger::RolePermissionUpdated),
            "role.deleted" => Ok(WebhookTrigger::RoleDeleted),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webhook_trigger_display_and_try_from_round_trip() {
        let cases = vec![
            (WebhookTrigger::UserCreated, "user.created"),
            (WebhookTrigger::UserUpdated, "user.updated"),
            (WebhookTrigger::UserDeleted, "user.deleted"),
            (WebhookTrigger::UserRoleAssigned, "user.role.assigned"),
            (WebhookTrigger::UserRoleUnassigned, "user.role.unassigned"),
            (WebhookTrigger::UserBulkDeleted, "user.bulk_deleted"),
            (
                WebhookTrigger::UserDeleteCredentials,
                "user.credentials.deleted",
            ),
            (WebhookTrigger::AuthResetPassword, "auth.reset_password"),
            (WebhookTrigger::ClientCreated, "client.created"),
            (WebhookTrigger::ClientUpdated, "client.updated"),
            (WebhookTrigger::ClientDeleted, "client.deleted"),
            (WebhookTrigger::ClientRoleCreated, "client.role.created"),
            (WebhookTrigger::ClientRoleUpdated, "client.role.updated"),
            (WebhookTrigger::RedirectUriCreated, "redirect_uri.created"),
            (WebhookTrigger::RedirectUriUpdated, "redirect_uri.updated"),
            (WebhookTrigger::RedirectUriDeleted, "redirect_uri.deleted"),
            (WebhookTrigger::RoleCreated, "role.created"),
            (WebhookTrigger::RoleUpdated, "role.updated"),
            (WebhookTrigger::RoleDeleted, "role.deleted"),
            (
                WebhookTrigger::RolePermissionUpdated,
                "role.permission.updated",
            ),
            (WebhookTrigger::RealmCreated, "realm.created"),
            (WebhookTrigger::RealmUpdated, "realm.updated"),
            (WebhookTrigger::RealmDeleted, "realm.deleted"),
            (
                WebhookTrigger::RealmSettingsUpdated,
                "realm.settings.updated",
            ),
            (WebhookTrigger::WebhookCreated, "webhook.created"),
            (WebhookTrigger::WebhookUpdated, "webhook.updated"),
            (WebhookTrigger::WebhookDeleted, "webhook.deleted"),
        ];

        for (variant, expected) in cases {
            assert_eq!(variant.to_string(), expected);
            assert_eq!(
                WebhookTrigger::try_from(expected.to_string()),
                Ok(variant.clone())
            );
        }
    }

    #[test]
    fn webhook_trigger_try_from_rejects_unknown() {
        let result = WebhookTrigger::try_from("unknown.event".to_string());
        assert!(result.is_err());
    }
}
