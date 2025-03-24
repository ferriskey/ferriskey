use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl UserSession {
    pub fn new(
        user_id: Uuid,
        realm_id: Uuid,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            realm_id,
            user_agent,
            ip_address,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(1),
        }
    }
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
