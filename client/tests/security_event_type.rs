use ferriskey_client::models::{GetSecurityEventsResponse, SecurityEventType};

const WIRE_NAMES: [&str; 33] = [
    "login_success",
    "login_failure",
    "password_reset",
    "password_reset_requested",
    "password_reset_completed",
    "password_imported",
    "user_created",
    "user_email_verified",
    "user_deleted",
    "role_assigned",
    "role_unassigned",
    "role_created",
    "role_removed",
    "role_updated",
    "role_permission_updated",
    "client_created",
    "client_deleted",
    "client_secret_rotated",
    "client_secret_viewed",
    "realm_config_changed",
    "email_not_sent",
    "email_sent",
    "client_maintenance_enabled",
    "client_maintenance_disabled",
    "session_created",
    "session_revoked",
    "identity_provider_link_removed",
    "webhook_delivery_exhausted",
    "token_exchanged",
    "account_selected",
    "account_selection_denied",
    "device_session_created",
    "unknown",
];

#[test]
fn every_server_event_type_deserializes_and_round_trips() {
    for name in WIRE_NAMES {
        let parsed: SecurityEventType = serde_json::from_str(&format!("\"{name}\""))
            .unwrap_or_else(|e| panic!("{name} must deserialize: {e}"));

        assert_eq!(parsed.to_string(), name);
        assert_eq!(
            serde_json::to_string(&parsed).expect("serialize"),
            format!("\"{name}\"")
        );
    }
}

#[test]
fn the_account_chooser_events_deserialize() {
    for (name, expected) in [
        ("account_selected", SecurityEventType::AccountSelected),
        (
            "account_selection_denied",
            SecurityEventType::AccountSelectionDenied,
        ),
        (
            "device_session_created",
            SecurityEventType::DeviceSessionCreated,
        ),
    ] {
        let parsed: SecurityEventType =
            serde_json::from_str(&format!("\"{name}\"")).expect("a known event type");

        assert_eq!(parsed, expected);
    }
}

#[test]
fn a_page_of_security_events_with_the_new_types_deserializes() {
    let event = |event_type: &str| {
        serde_json::json!({
            "id": "0191a0a0-0000-7000-8000-000000000001",
            "realm_id": "0191a0a0-0000-7000-8000-000000000002",
            "event_type": event_type,
            "status": "success",
            "timestamp": "2026-11-08T10:00:00Z",
            "target_type": "session",
            "details": { "via": "account_chooser" }
        })
    };
    let page = serde_json::json!({
        "data": [
            event("account_selected"),
            event("account_selection_denied"),
            event("device_session_created"),
        ]
    });

    let parsed: GetSecurityEventsResponse = serde_json::from_value(page).expect("a returned page");

    let types: Vec<SecurityEventType> = parsed.data.iter().map(|e| e.event_type).collect();
    assert_eq!(
        types,
        vec![
            SecurityEventType::AccountSelected,
            SecurityEventType::AccountSelectionDenied,
            SecurityEventType::DeviceSessionCreated,
        ]
    );
}
