use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

impl EmailVerificationToken {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_used(&self) -> bool {
        self.used_at.is_some()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.is_used()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_is_expired_returns_false_for_future_expiration() {
        let token = EmailVerificationToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            token_hash: "hash123".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            used_at: None,
        };

        assert!(!token.is_expired());
    }

    #[test]
    fn test_is_expired_returns_true_for_past_expiration() {
        let token = EmailVerificationToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            token_hash: "hash123".to_string(),
            expires_at: Utc::now() - Duration::hours(1),
            created_at: Utc::now() - Duration::hours(2),
            used_at: None,
        };

        assert!(token.is_expired());
    }

    #[test]
    fn test_is_used_returns_true_when_used_at_is_some() {
        let token = EmailVerificationToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            token_hash: "hash123".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            used_at: Some(Utc::now()),
        };

        assert!(token.is_used());
    }

    #[test]
    fn test_is_used_returns_false_when_used_at_is_none() {
        let token = EmailVerificationToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            token_hash: "hash123".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            used_at: None,
        };

        assert!(!token.is_used());
    }

    #[test]
    fn test_is_valid_returns_true_for_unused_and_not_expired() {
        let token = EmailVerificationToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            token_hash: "hash123".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            used_at: None,
        };

        assert!(token.is_valid());
    }

    #[test]
    fn test_is_valid_returns_false_for_expired_token() {
        let token = EmailVerificationToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            token_hash: "hash123".to_string(),
            expires_at: Utc::now() - Duration::hours(1),
            created_at: Utc::now() - Duration::hours(2),
            used_at: None,
        };

        assert!(!token.is_valid());
    }

    #[test]
    fn test_is_valid_returns_false_for_used_token() {
        let token = EmailVerificationToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            token_hash: "hash123".to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Utc::now(),
            used_at: Some(Utc::now()),
        };

        assert!(!token.is_valid());
    }

    #[test]
    fn test_is_valid_returns_false_for_expired_and_used_token() {
        let token = EmailVerificationToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            token_hash: "hash123".to_string(),
            expires_at: Utc::now() - Duration::hours(1),
            created_at: Utc::now() - Duration::hours(2),
            used_at: Some(Utc::now() - Duration::minutes(30)),
        };

        assert!(!token.is_valid());
    }
}
