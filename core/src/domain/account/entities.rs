use chrono::{DateTime, Utc};
use thiserror::Error;
use uuid::Uuid;

pub struct AccountHint {
    pub user_id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub last_used_at: DateTime<Utc>,
}

impl AccountHint {
    pub fn new(user_id: Uuid, display_name: String, avatar_url: Option<String>) -> Self {
        Self {
            user_id,
            display_name,
            avatar_url,
            last_used_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum AccountError {
    #[error("Account hints not found")]
    NotFound,

    #[error("Internal server error")]
    InternalServerError,

    #[error("Forbidden")]
    Forbidden,

    #[error("Account not found")]
    AccountNotFound,
}
