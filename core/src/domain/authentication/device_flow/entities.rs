use std::fmt::Display;

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::domain::common::generate_uuid_v7;
use crate::domain::realm::entities::RealmId;

/// Charset used to render the end-user code (RFC 8628 §6.1).
///
/// A 20-character (base20) alphabet that excludes visually ambiguous
/// characters (vowels, `0`/`O`, `1`/`I`, etc.) so codes are easy to read
/// aloud and type on a constrained device.
const USER_CODE_CHARSET: &[u8] = b"BCDFGHJKLMNPQRSTVWXZ";

/// Number of significant characters in a user code (excluding the separator).
const USER_CODE_LENGTH: usize = 8;

/// The human-readable code the user types on the verification page.
///
/// Rendered in the `XXXX-XXXX` format mandated by RFC 8628 §6.1.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserCode(String);

impl UserCode {
    /// Generate a fresh RFC 8628 §6.1 compliant user code.
    ///
    /// Produces 8 characters drawn from [`USER_CODE_CHARSET`] grouped as
    /// `XXXX-XXXX`.
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let chars: String = (0..USER_CODE_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..USER_CODE_CHARSET.len());
                USER_CODE_CHARSET[idx] as char
            })
            .collect();

        Self(format!("{}-{}", &chars[0..4], &chars[4..USER_CODE_LENGTH]))
    }

    /// Wrap an existing code (e.g. when reconstructing from persistence).
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Display for UserCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Lifecycle state of a device authorization session (RFC 8628).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceAuthStatus {
    /// Awaiting end-user approval — the token endpoint returns
    /// `authorization_pending`.
    #[default]
    #[serde(rename = "pending")]
    Pending,

    /// The end user approved the request; tokens can now be issued.
    #[serde(rename = "approved")]
    Approved,

    /// The end user denied the request — the token endpoint returns
    /// `access_denied`.
    #[serde(rename = "denied")]
    Denied,

    /// The session lifetime elapsed before approval — the token endpoint
    /// returns `expired_token`.
    #[serde(rename = "expired")]
    Expired,
}

impl DeviceAuthStatus {
    /// Canonical string stored in the database / used by `Display`.
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceAuthStatus::Pending => "pending",
            DeviceAuthStatus::Approved => "approved",
            DeviceAuthStatus::Denied => "denied",
            DeviceAuthStatus::Expired => "expired",
        }
    }

    /// Parse a status back from its persisted string form.
    pub fn from_db_value(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(DeviceAuthStatus::Pending),
            "approved" => Some(DeviceAuthStatus::Approved),
            "denied" => Some(DeviceAuthStatus::Denied),
            "expired" => Some(DeviceAuthStatus::Expired),
            _ => None,
        }
    }
}

impl Display for DeviceAuthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A device authorization session created by the device authorization
/// endpoint and polled by the device until the user approves or denies it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthSession {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub client_id: Uuid,
    /// Opaque code the device polls with (RFC 8628 `device_code`).
    pub device_code: Uuid,
    /// Human-readable code the user enters on the verification page.
    pub user_code: UserCode,
    pub scope: Option<String>,
    pub status: DeviceAuthStatus,
    /// Set once the user approves the request.
    pub user_id: Option<Uuid>,
    /// Minimum polling interval, in seconds (RFC 8628 `interval`).
    pub interval: i64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// Timestamp of the device's last poll, used for `slow_down` enforcement.
    pub last_polled_at: Option<DateTime<Utc>>,
}

/// Input parameters for [`DeviceAuthSession::new`].
pub struct DeviceAuthSessionConfig {
    pub realm_id: RealmId,
    pub client_id: Uuid,
    pub scope: Option<String>,
    /// Minimum polling interval, in seconds.
    pub interval: i64,
    /// Session lifetime, in seconds.
    pub expires_in: i64,
}

/// Non-sensitive projection of a [`DeviceAuthSession`] sent as the body of
/// `auth.device_flow.*` webhooks. Deliberately excludes the `user_code` so it
/// can never leak through a webhook subscriber.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFlowEventPayload {
    pub session_id: Uuid,
    pub realm_id: Uuid,
    pub client_id: Uuid,
    pub status: DeviceAuthStatus,
    /// Full device code on `Initiated` (already returned to the device in the
    /// HTTP response); SHA-256 hex for all other events to avoid leaking it.
    pub device_code: String,
    pub user_id: Option<Uuid>,
}

/// SHA-256 hex digest of a device code UUID's raw bytes.
fn hash_device_code(device_code: &Uuid) -> String {
    hex::encode(Sha256::digest(device_code.as_bytes()))
}

impl DeviceFlowEventPayload {
    pub fn new(session: &DeviceAuthSession, reveal_device_code: bool) -> Self {
        let device_code = if reveal_device_code {
            session.device_code.to_string()
        } else {
            hash_device_code(&session.device_code)
        };

        Self {
            session_id: session.id,
            realm_id: session.realm_id.into(),
            client_id: session.client_id,
            status: session.status,
            device_code,
            user_id: session.user_id,
        }
    }
}

impl DeviceAuthSession {
    pub fn new(config: DeviceAuthSessionConfig) -> Self {
        let now = Utc::now();

        Self {
            id: generate_uuid_v7(),
            realm_id: config.realm_id,
            client_id: config.client_id,
            device_code: generate_uuid_v7(),
            user_code: UserCode::generate(),
            scope: config.scope,
            status: DeviceAuthStatus::Pending,
            user_id: None,
            interval: config.interval,
            created_at: now,
            expires_at: now + Duration::seconds(config.expires_in),
            last_polled_at: None,
        }
    }

    /// Whether the session lifetime has elapsed.
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_session() -> DeviceAuthSession {
        DeviceAuthSession::new(DeviceAuthSessionConfig {
            realm_id: RealmId::from(Uuid::new_v4()),
            client_id: Uuid::new_v4(),
            scope: None,
            interval: 5,
            expires_in: 600,
        })
    }

    #[test]
    fn payload_initiated_reveals_full_device_code() {
        let session = test_session();
        let payload = DeviceFlowEventPayload::new(&session, true);

        assert_eq!(payload.device_code, session.device_code.to_string());
        assert_eq!(payload.session_id, session.id);
        assert_eq!(payload.realm_id, Uuid::from(session.realm_id));
        assert_eq!(payload.client_id, session.client_id);
        assert_eq!(payload.status, session.status);
        assert_eq!(payload.user_id, None);
    }

    #[test]
    fn payload_initiated_carries_user_id_when_set() {
        let mut session = test_session();
        let user_id = Uuid::new_v4();
        session.user_id = Some(user_id);

        let payload = DeviceFlowEventPayload::new(&session, true);
        assert_eq!(payload.user_id, Some(user_id));
    }

    #[test]
    fn payload_non_initiated_hashes_device_code() {
        let session = test_session();
        let payload = DeviceFlowEventPayload::new(&session, false);

        let raw = session.device_code.to_string();
        assert_ne!(payload.device_code, raw);

        // Must be a 64-char lowercase hex string (SHA-256).
        assert_eq!(payload.device_code.len(), 64);
        assert!(
            payload
                .device_code
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_uppercase())
        );

        // Must equal the expected SHA-256 hex of the UUID bytes.
        let expected = hex::encode(sha2::Sha256::digest(session.device_code.as_bytes()));
        assert_eq!(payload.device_code, expected);
    }

    #[test]
    fn payload_denied_session_with_user_id_carries_through() {
        let mut session = test_session();
        session.status = DeviceAuthStatus::Denied;
        let user_id = Uuid::new_v4();
        session.user_id = Some(user_id);

        let payload = DeviceFlowEventPayload::new(&session, false);

        assert_eq!(payload.user_id, Some(user_id));
        assert_eq!(payload.status, DeviceAuthStatus::Denied);
        // device_code is hashed (64 hex chars), not the raw UUID string.
        assert_ne!(payload.device_code, session.device_code.to_string());
        assert_eq!(payload.device_code.len(), 64);
    }

    #[test]
    fn user_code_has_expected_format() {
        let code = UserCode::generate();
        let value = code.as_str();

        // `XXXX-XXXX`: 8 significant chars plus a single separator.
        assert_eq!(value.len(), USER_CODE_LENGTH + 1);
        assert_eq!(value.as_bytes()[4], b'-');

        let groups: Vec<&str> = value.split('-').collect();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 4);
        assert_eq!(groups[1].len(), 4);
    }

    #[test]
    fn user_code_only_uses_the_allowed_charset() {
        for _ in 0..1_000 {
            let code = UserCode::generate();
            for byte in code.as_str().bytes().filter(|&b| b != b'-') {
                assert!(
                    USER_CODE_CHARSET.contains(&byte),
                    "char {} not in charset",
                    byte as char
                );
            }
        }
    }

    #[test]
    fn user_codes_are_not_all_identical() {
        // Sanity check the generator is actually random, not a constant.
        let first = UserCode::generate();
        let differs = (0..50).any(|_| UserCode::generate() != first);
        assert!(differs);
    }
}
