use chrono::{DateTime, Duration, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::realm::RealmId;
use crate::realm::scope::RealmOwned;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SessionState {
    Active,
    SoftExpired,
    Expired,
}

#[derive(Debug, Clone)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub soft_expiry_duration: Option<Duration>,
    pub sso_token_hash: Option<String>,
    /// Whether the SSO cookie outlives the browser ("remember me").
    pub persistent: bool,
    /// When the user last authenticated interactively in this session.
    pub authenticated_at: DateTime<Utc>,
}

impl UserSession {
    pub fn new(
        user_id: Uuid,
        realm_id: Uuid,
        user_agent: Option<String>,
        ip_address: Option<String>,
        session_duration: Duration,
        soft_expiry_duration: Option<Duration>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            user_id,
            realm_id,
            user_agent,
            ip_address,
            created_at: now,
            expires_at: now + session_duration,
            last_seen_at: None,
            soft_expiry_duration,
            sso_token_hash: None,
            persistent: false,
            authenticated_at: now,
        }
    }

    /// The `Max-Age` of the SSO cookie, in seconds: the remaining lifetime of a
    /// persistent session, or `None` for a cookie that dies with the browser.
    pub fn cookie_max_age(&self) -> Option<i64> {
        self.persistent
            .then(|| (self.expires_at - Utc::now()).num_seconds().max(0))
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn get_state(&self) -> SessionState {
        let now = Utc::now();

        if now > self.expires_at {
            SessionState::Expired
        } else if let Some(soft_expiry_duration) = self.soft_expiry_duration {
            if now > self.expires_at - soft_expiry_duration {
                SessionState::SoftExpired
            } else {
                SessionState::Active
            }
        } else {
            SessionState::Active
        }
    }
}

impl RealmOwned for UserSession {
    fn realm_id(&self) -> RealmId {
        RealmId::new(self.realm_id)
    }
}

impl SessionState {
    // Returns the string representation of the session state
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::SoftExpired => "soft-expired",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Session not found")]
    NotFound,
    #[error("Session expired")]
    Expired,
    #[error("Session is invalid")]
    Invalid,
    #[error("Failed to create session")]
    CreateError,
    #[error("Failed to delete session")]
    DeleteError,
    #[error("Failed to update session")]
    UpdateError,
}
