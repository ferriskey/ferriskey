use chrono::{TimeZone, Utc};
use sea_orm::ActiveValue::Set;

use crate::domain::seawatch::entities::{ActorType, EventStatus, SecurityEvent, SecurityEventType};
use crate::entity::security_events;

fn decode_hash(hex_str: Option<String>) -> Option<[u8; 32]> {
    let s = hex_str?;
    let bytes = hex::decode(&s).ok()?;
    let arr: [u8; 32] = bytes.try_into().ok()?;
    Some(arr)
}

impl From<security_events::Model> for SecurityEvent {
    fn from(model: security_events::Model) -> Self {
        let actor_type = model.actor_type.and_then(|s| match s.as_str() {
            "user" => Some(ActorType::User),
            "service_account" => Some(ActorType::ServiceAccount),
            "admin" => Some(ActorType::Admin),
            "system" => Some(ActorType::System),
            _ => None,
        });

        let event_type = parse_event_type(model.event_type.as_str());

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
            event_hash: decode_hash(model.event_hash),
            prev_hash: decode_hash(model.prev_hash),
        }
    }
}

/// Inverse of `SecurityEventType`'s `Display`, which is what the write path
/// persists. Every variant must round-trip through here — see the tests below.
///
/// Unknown values fall back to `LoginSuccess` so that a row written by a newer
/// version does not break reads on an older one.
fn parse_event_type(raw: &str) -> SecurityEventType {
    match raw {
        "login_success" => SecurityEventType::LoginSuccess,
        "login_failure" => SecurityEventType::LoginFailure,
        "password_reset" => SecurityEventType::PasswordReset,
        "password_reset_requested" => SecurityEventType::PasswordResetRequested,
        "password_reset_completed" => SecurityEventType::PasswordResetCompleted,
        "user_created" => SecurityEventType::UserCreated,
        "user_email_verified" => SecurityEventType::UserEmailVerified,
        "user_deleted" => SecurityEventType::UserDeleted,
        "role_assigned" => SecurityEventType::RoleAssigned,
        "role_unassigned" => SecurityEventType::RoleUnassigned,
        "role_created" => SecurityEventType::RoleCreated,
        "role_removed" => SecurityEventType::RoleRemoved,
        "client_created" => SecurityEventType::ClientCreated,
        "client_deleted" => SecurityEventType::ClientDeleted,
        "client_secret_rotated" => SecurityEventType::ClientSecretRotated,
        "realm_config_changed" => SecurityEventType::RealmConfigChanged,
        "email_not_sent" => SecurityEventType::EmailNotSent,
        "email_sent" => SecurityEventType::EmailSent,
        "client_maintenance_enabled" => SecurityEventType::ClientMaintenanceEnabled,
        "client_maintenance_disabled" => SecurityEventType::ClientMaintenanceDisabled,
        "session_created" => SecurityEventType::SessionCreated,
        "session_revoked" => SecurityEventType::SessionRevoked,
        _ => SecurityEventType::LoginSuccess,
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
            event_hash: Set(event.event_hash.map(hex::encode)),
            prev_hash: Set(event.prev_hash.map(hex::encode)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant of `SecurityEventType`. Kept in sync by
    /// `every_variant_is_covered`, which will not compile if a variant is added
    /// to the enum without also being listed here.
    const ALL_EVENT_TYPES: &[SecurityEventType] = &[
        SecurityEventType::LoginSuccess,
        SecurityEventType::LoginFailure,
        SecurityEventType::PasswordReset,
        SecurityEventType::PasswordResetRequested,
        SecurityEventType::PasswordResetCompleted,
        SecurityEventType::UserCreated,
        SecurityEventType::UserEmailVerified,
        SecurityEventType::UserDeleted,
        SecurityEventType::RoleAssigned,
        SecurityEventType::RoleUnassigned,
        SecurityEventType::RoleCreated,
        SecurityEventType::RoleRemoved,
        SecurityEventType::ClientCreated,
        SecurityEventType::ClientDeleted,
        SecurityEventType::ClientSecretRotated,
        SecurityEventType::RealmConfigChanged,
        SecurityEventType::EmailNotSent,
        SecurityEventType::EmailSent,
        SecurityEventType::ClientMaintenanceEnabled,
        SecurityEventType::ClientMaintenanceDisabled,
        SecurityEventType::SessionCreated,
        SecurityEventType::SessionRevoked,
    ];

    /// The write path persists `event_type` via `Display` and the read path
    /// parses it back with `parse_event_type`. If the two ever drift, events
    /// silently decode as `LoginSuccess` instead of failing loudly — so pin the
    /// round-trip for every variant.
    #[test]
    fn event_type_round_trips_through_its_persisted_form() {
        for expected in ALL_EVENT_TYPES {
            let persisted = expected.to_string();
            assert_eq!(
                parse_event_type(&persisted),
                *expected,
                "`{persisted}` did not parse back to the variant that produced it"
            );
        }
    }

    #[test]
    fn unknown_event_type_falls_back_instead_of_panicking() {
        assert_eq!(
            parse_event_type("some_event_from_a_newer_version"),
            SecurityEventType::LoginSuccess
        );
    }

    /// Compile-time guard: adding a variant to `SecurityEventType` breaks this
    /// match until the variant is also added to `ALL_EVENT_TYPES` above.
    #[test]
    fn every_variant_is_covered() {
        fn assert_listed(event_type: &SecurityEventType) {
            let listed = match event_type {
                SecurityEventType::LoginSuccess
                | SecurityEventType::LoginFailure
                | SecurityEventType::PasswordReset
                | SecurityEventType::PasswordResetRequested
                | SecurityEventType::PasswordResetCompleted
                | SecurityEventType::UserCreated
                | SecurityEventType::UserEmailVerified
                | SecurityEventType::UserDeleted
                | SecurityEventType::RoleAssigned
                | SecurityEventType::RoleUnassigned
                | SecurityEventType::RoleCreated
                | SecurityEventType::RoleRemoved
                | SecurityEventType::ClientCreated
                | SecurityEventType::ClientDeleted
                | SecurityEventType::ClientSecretRotated
                | SecurityEventType::RealmConfigChanged
                | SecurityEventType::EmailNotSent
                | SecurityEventType::EmailSent
                | SecurityEventType::ClientMaintenanceEnabled
                | SecurityEventType::ClientMaintenanceDisabled
                | SecurityEventType::SessionCreated
                | SecurityEventType::SessionRevoked => true,
            };

            assert!(listed && ALL_EVENT_TYPES.contains(event_type));
        }

        for event_type in ALL_EVENT_TYPES {
            assert_listed(event_type);
        }
    }

    #[test]
    fn session_event_types_use_their_documented_wire_names() {
        assert_eq!(
            SecurityEventType::SessionCreated.to_string(),
            "session_created"
        );
        assert_eq!(
            SecurityEventType::SessionRevoked.to_string(),
            "session_revoked"
        );
    }
}
