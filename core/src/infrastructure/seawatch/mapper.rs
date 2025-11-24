use chrono::{TimeZone, Utc};
use sea_orm::ActiveValue::Set;

use crate::domain::seawatch::entities::{ActorType, EventStatus, SecurityEvent, SecurityEventType};
use crate::entity::security_events;

impl From<security_events::Model> for SecurityEvent {
    fn from(model: security_events::Model) -> Self {
        let actor_type = model.actor_type.and_then(|s| match s.as_str() {
            "user" => Some(ActorType::User),
            "service_account" => Some(ActorType::ServiceAccount),
            "admin" => Some(ActorType::Admin),
            "system" => Some(ActorType::System),
            _ => None,
        });

        let event_type = match model.event_type.as_str() {
            "login_success" => SecurityEventType::LoginSuccess,
            "login_failure" => SecurityEventType::LoginFailure,
            "password_reset" => SecurityEventType::PasswordReset,
            "user_created" => SecurityEventType::UserCreated,
            "user_deleted" => SecurityEventType::UserDeleted,
            "role_assigned" => SecurityEventType::RoleAssigned,
            "role_unassigned" => SecurityEventType::RoleUnassigned,
            "role_created" => SecurityEventType::RoleCreated,
            "role_removed" => SecurityEventType::RoleRemoved,
            "client_created" => SecurityEventType::ClientCreated,
            "client_deleted" => SecurityEventType::ClientDeleted,
            "client_secret_rotated" => SecurityEventType::ClientSecretRotated,
            "realm_config_changed" => SecurityEventType::RealmConfigChanged,
            _ => SecurityEventType::LoginSuccess,
        };

        let status = match model.status.as_str() {
            "success" => EventStatus::Success,
            "failure" => EventStatus::Failure,
            _ => EventStatus::Success,
        };

        SecurityEvent {
            id: model.id.into(),
            realm_id: model.realm_id.into(),
            actor_id: model.actor_id,
            actor_type,
            event_type,
            status,
            target_type: model.target_type,
            target_id: model.target_id,
            resource: model.resource,
            timestamp: Utc.from_utc_datetime(&model.timestamp),
            trace_id: model.trace_id,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
            details: model.details,
        }
    }
}

impl From<SecurityEvent> for security_events::ActiveModel {
    fn from(event: SecurityEvent) -> Self {
        security_events::ActiveModel {
            id: Set(event.id.into()),
            realm_id: Set(event.realm_id.into()),
            actor_id: Set(event.actor_id),
            actor_type: Set(event.actor_type.map(|t| t.to_string())),
            event_type: Set(event.event_type.to_string()),
            status: Set(event.status.to_string()),
            target_type: Set(event.target_type),
            target_id: Set(event.target_id),
            resource: Set(event.resource),
            timestamp: Set(event.timestamp.naive_utc()),
            trace_id: Set(event.trace_id),
            ip_address: Set(event.ip_address),
            user_agent: Set(event.user_agent),
            details: Set(event.details),
            created_at: Set(Utc::now().naive_utc()),
        }
    }
}
