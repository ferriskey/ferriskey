//! OIDC Back-Channel Logout 1.0: when a session ends, FerrisKey POSTs a
//! signed logout token to every client that took part in it.

use std::time::Duration;

use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::realm::entities::RealmId;

/// The `events` member every logout token carries (§2.4).
pub const BACKCHANNEL_LOGOUT_EVENT: &str = "http://schemas.openid.net/event/backchannel-logout";

/// The JWT `typ` header of a logout token (§2.4).
pub const LOGOUT_TOKEN_TYP: &str = "logout+jwt";

/// A user session that just ended, for any reason but expiry, with the
/// clients that took part in it. The participants are read before the session
/// is deleted: deleting a user cascades to its access tokens, so they cannot
/// be read back once the worker picks the session up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndedSession {
    pub realm_id: RealmId,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub participants: Vec<SessionParticipant>,
}

/// A client that received a token in a session, with the issuer that token
/// named. The issuer is taken from the token because a background task has no
/// request to derive the realm URL from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionParticipant {
    pub client_id: String,
    pub issuer: String,
}

/// The claims of a logout token (§2.4). It carries both `sub` and `sid`, and
/// never a `nonce`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LogoutTokenClaims {
    pub iss: String,
    pub sub: Uuid,
    pub aud: String,
    pub iat: i64,
    pub exp: i64,
    pub jti: Uuid,
    pub sid: Uuid,
    pub events: serde_json::Value,
}

impl LogoutTokenClaims {
    pub fn new(
        participant: &SessionParticipant,
        session: &EndedSession,
        now: i64,
        lifetime: Duration,
    ) -> Self {
        Self {
            iss: participant.issuer.clone(),
            sub: session.user_id,
            aud: participant.client_id.clone(),
            iat: now,
            exp: now + lifetime.as_secs() as i64,
            jti: ferriskey_domain::generate_uuid_v7(),
            sid: session.session_id,
            events: json!({ BACKCHANNEL_LOGOUT_EVENT: {} }),
        }
    }
}

/// How hard FerrisKey tries to reach a client, and how long its logout
/// tokens stay valid.
#[derive(Debug, Clone)]
pub struct BackchannelLogoutConfig {
    /// The wait before each attempt; its length is the number of attempts.
    pub attempt_delays: Vec<Duration>,
    pub token_lifetime: Duration,
}

impl Default for BackchannelLogoutConfig {
    fn default() -> Self {
        Self {
            attempt_delays: vec![
                Duration::ZERO,
                Duration::from_secs(2),
                Duration::from_secs(10),
            ],
            token_lifetime: Duration::from_secs(120),
        }
    }
}

/// Why a logout token did not reach a client.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LogoutDeliveryError {
    /// The endpoint answered with an error status. A 4xx is the client
    /// refusing the token (§2.8) and is not retried.
    #[error("the endpoint answered {0}")]
    Rejected(u16),

    /// The endpoint resolves only to addresses this deployment may not reach.
    #[error("the endpoint resolves to a forbidden address")]
    ForbiddenAddress,

    /// The endpoint is not a usable URL.
    #[error("the endpoint is not a usable URL")]
    InvalidEndpoint,

    /// DNS, connection or timeout failure.
    #[error("transport failure: {0}")]
    Transport(String),
}

impl LogoutDeliveryError {
    /// Whether another attempt could succeed.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Rejected(status) => *status >= 500,
            Self::Transport(_) => true,
            Self::ForbiddenAddress | Self::InvalidEndpoint => false,
        }
    }
}

/// What happened to one session's logout tokens.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DeliveryReport {
    pub delivered: Vec<String>,
    pub failed: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_logout_token_names_the_session_the_user_and_the_event() {
        let session = EndedSession {
            realm_id: RealmId::default(),
            session_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            participants: Vec::new(),
        };
        let participant = SessionParticipant {
            client_id: "orders".to_string(),
            issuer: "https://sso.example.com/realms/home".to_string(),
        };

        let claims =
            LogoutTokenClaims::new(&participant, &session, 1_000, Duration::from_secs(120));
        let json = serde_json::to_value(&claims).expect("claims serialize");

        assert_eq!(json["iss"], "https://sso.example.com/realms/home");
        assert_eq!(json["aud"], "orders");
        assert_eq!(json["sub"], json!(session.user_id));
        assert_eq!(json["sid"], json!(session.session_id));
        assert_eq!(json["iat"], 1_000);
        assert_eq!(json["exp"], 1_120);
        assert_eq!(json["events"], json!({ BACKCHANNEL_LOGOUT_EVENT: {} }));
        assert!(
            json.get("nonce").is_none(),
            "a logout token never carries a nonce"
        );
    }

    #[test]
    fn only_server_errors_and_transport_failures_are_retried() {
        assert!(LogoutDeliveryError::Transport("timeout".into()).is_retryable());
        assert!(LogoutDeliveryError::Rejected(503).is_retryable());
        assert!(!LogoutDeliveryError::Rejected(400).is_retryable());
        assert!(!LogoutDeliveryError::ForbiddenAddress.is_retryable());
        assert!(!LogoutDeliveryError::InvalidEndpoint.is_retryable());
    }
}
