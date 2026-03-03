use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::realm::entities::RealmId;

#[derive(Debug, Clone)]
pub struct AccountHint {
    pub user_id: Uuid,
    pub realm_id: RealmId,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub last_used_at: DateTime<Utc>,
}

impl AccountHint {
    pub fn new(
        user_id: Uuid,
        realm_id: RealmId,
        display_name: String,
        avatar_url: Option<String>,
    ) -> Self {
        Self {
            user_id,
            realm_id,
            display_name,
            avatar_url,
            last_used_at: Utc::now(),
        }
    }
}
