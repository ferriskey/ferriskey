use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct RefreshToken {
    pub id: Uuid,
    pub jti: Uuid,
    pub user_id: Uuid,
    pub revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl RefreshToken {
    pub fn new(
        id: Uuid,
        jti: Uuid,
        user_id: Uuid,
        revoked: bool,
        expires_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            jti,
            user_id,
            revoked,
            expires_at,
            created_at,
        }
    }
}
