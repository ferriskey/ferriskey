pub use ferriskey_trident::entities::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MagicLink {
    pub id: Uuid,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub token: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl MagicLink {
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}