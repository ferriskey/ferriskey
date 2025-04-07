use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Error)]
pub enum AuthSessionError {
    #[error("Session not found")]
    NotFound,
    #[error("Session already exists")]
    AlreadyExists,
    #[error("Session expired")]
    Expired,
    #[error("Session invalid")]
    Invalid,
    #[error("Session creation error")]
    CreateSessionError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub client_id: Uuid,
    pub redirect_uri: String,
    pub response_type: String,
    pub scope: String,
    pub state: Option<String>,
    pub nonce: Option<String>,
    pub user_id: Option<Uuid>,
    pub authenticated: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl AuthSession {
    pub fn new(
        realm_id: Uuid,
        client_id: Uuid,
        redirect_uri: String,
        response_type: String,
        scope: String,
        state: Option<String>,
        nonce: Option<String>,
        user_id: Option<Uuid>,
        authenticated: bool,
    ) -> Self {
        let now = Utc::now();
        let seconds = now.timestamp().try_into().unwrap_or(0);

        let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            client_id,
            redirect_uri,
            response_type,
            scope,
            state,
            nonce,
            user_id,
            authenticated,
            created_at: now,
            expires_at: now + Duration::minutes(10),
        }
    }
}
