use std::fmt::Display;

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
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

impl Display for DeviceAuthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceAuthStatus::Pending => write!(f, "pending"),
            DeviceAuthStatus::Approved => write!(f, "approved"),
            DeviceAuthStatus::Denied => write!(f, "denied"),
            DeviceAuthStatus::Expired => write!(f, "expired"),
        }
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
