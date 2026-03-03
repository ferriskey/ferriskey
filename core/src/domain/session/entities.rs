use chrono::{DateTime, Duration, Utc};
use sea_orm::EnumIter;
use thiserror::Error;
use uuid::Uuid;

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum SessionState {
    Active,
    SoftExpired,
    Expired,
}

pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub soft_expiry_duration: Option<Duration>,
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
        let expires_at = Utc::now() + session_duration;
        Self {
            id: Uuid::new_v4(),
            user_id,
            realm_id,
            user_agent,
            ip_address,
            created_at: Utc::now(),
            expires_at,
            soft_expiry_duration,
        }
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
}
