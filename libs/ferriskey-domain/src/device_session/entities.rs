use chrono::{DateTime, Duration, Utc};
use thiserror::Error;
use uuid::Uuid;

use crate::realm::RealmId;
use crate::realm::scope::RealmOwned;
use crate::session::entities::SessionState;
use crate::user::entities::User;

const MASKED_RUN: &str = "••••";
const DEVICE_LIFETIME_CAP_DAYS: i64 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceSession {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub device_token_hash: String,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl DeviceSession {
    pub fn new(
        realm_id: Uuid,
        device_token_hash: String,
        session_expires_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            realm_id,
            device_token_hash,
            created_at: now,
            last_seen_at: now,
            expires_at: Self::extended_expiry(None, session_expires_at, now),
        }
    }

    pub fn extended_expiry(
        current: Option<DateTime<Utc>>,
        session_expires_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> DateTime<Utc> {
        let longest = current.map_or(session_expires_at, |c| c.max(session_expires_at));

        longest.min(now + Duration::days(DEVICE_LIFETIME_CAP_DAYS))
    }

    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now >= self.expires_at
    }
}

impl RealmOwned for DeviceSession {
    fn realm_id(&self) -> RealmId {
        RealmId::new(self.realm_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountEntry {
    pub session_id: Uuid,
    pub display_name: String,
    pub masked_email: Option<String>,
    pub state: SessionState,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub requires_reauthentication: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedAccount {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub new_sso_secret: String,
    pub cookie_max_age: Option<i64>,
}

#[derive(Debug, Clone, Error)]
pub enum DeviceSessionError {
    #[error("Device session not found")]
    NotFound,
    #[error("Failed to create device session")]
    CreateError,
    #[error("Failed to update device session")]
    UpdateError,
    #[error("Failed to delete device session")]
    DeleteError,
}

pub fn display_name(user: &User) -> String {
    let part = |value: &Option<String>| {
        value
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
    };

    match (part(&user.firstname), part(&user.lastname)) {
        (Some(first), Some(last)) => format!("{first} {last}"),
        _ => user.username.clone(),
    }
}

pub fn mask_email(email: &str) -> String {
    let email = email.trim();

    if email.is_empty() {
        return String::new();
    }

    let Some((local, domain)) = email.rsplit_once('@') else {
        return MASKED_RUN.to_owned();
    };

    match local.chars().next() {
        Some(first) => format!("{first}{MASKED_RUN}@{domain}"),
        None => format!("{MASKED_RUN}@{domain}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm::RealmId;

    fn user(username: &str, firstname: Option<&str>, lastname: Option<&str>) -> User {
        let now = Utc::now();
        User {
            id: Uuid::new_v4(),
            realm_id: RealmId::new(Uuid::new_v4()),
            client_id: None,
            username: username.to_owned(),
            firstname: firstname.map(str::to_owned),
            lastname: lastname.map(str::to_owned),
            email: None,
            email_verified: false,
            enabled: true,
            roles: None,
            realm: None,
            required_actions: Vec::new(),
            created_at: now,
            updated_at: now,
            failed_login_attempts: 0,
            locked_until: None,
            locale: None,
            federation_provider_id: None,
        }
    }

    #[test]
    fn display_name_joins_first_and_last_name() {
        assert_eq!(
            display_name(&user("alice", Some("Alice"), Some("Doe"))),
            "Alice Doe"
        );
    }

    #[test]
    fn display_name_falls_back_to_username_without_a_full_name() {
        assert_eq!(display_name(&user("alice", None, None)), "alice");
        assert_eq!(display_name(&user("alice", Some("Alice"), None)), "alice");
        assert_eq!(display_name(&user("alice", None, Some("Doe"))), "alice");
    }

    #[test]
    fn display_name_ignores_blank_name_parts() {
        assert_eq!(
            display_name(&user("alice", Some("  "), Some("Doe"))),
            "alice"
        );
        assert_eq!(display_name(&user("alice", Some(""), Some(""))), "alice");
    }

    #[test]
    fn display_name_trims_the_name_parts() {
        assert_eq!(
            display_name(&user("alice", Some(" Alice "), Some(" Doe "))),
            "Alice Doe"
        );
    }

    #[test]
    fn mask_email_keeps_the_first_character_and_the_domain() {
        assert_eq!(mask_email("alice@example.com"), "a••••@example.com");
    }

    #[test]
    fn mask_email_never_reveals_more_than_the_first_character() {
        assert_eq!(mask_email("a@example.com"), "a••••@example.com");
        assert_eq!(mask_email("é.long.name@example.com"), "é••••@example.com");
    }

    #[test]
    fn mask_email_without_at_sign_reveals_nothing() {
        assert_eq!(mask_email("not-an-email"), "••••");
    }

    #[test]
    fn mask_email_without_local_part_reveals_only_the_domain() {
        assert_eq!(mask_email("@example.com"), "••••@example.com");
    }

    #[test]
    fn mask_email_of_empty_input_is_empty() {
        assert_eq!(mask_email(""), "");
        assert_eq!(mask_email("   "), "");
    }

    #[test]
    fn mask_email_splits_on_the_last_at_sign() {
        assert_eq!(mask_email("a@b@example.com"), "a••••@example.com");
    }

    #[test]
    fn device_lifetime_follows_the_longest_session() {
        let now = Utc::now();
        let session = now + Duration::days(2);
        let current = now + Duration::days(1);

        assert_eq!(
            DeviceSession::extended_expiry(Some(current), session, now),
            session
        );
        assert_eq!(
            DeviceSession::extended_expiry(Some(session), current, now),
            session
        );
        assert_eq!(DeviceSession::extended_expiry(None, session, now), session);
    }

    #[test]
    fn device_lifetime_is_capped_at_thirty_days() {
        let now = Utc::now();
        let far = now + Duration::days(400);

        assert_eq!(
            DeviceSession::extended_expiry(None, far, now),
            now + Duration::days(30)
        );
        assert_eq!(
            DeviceSession::extended_expiry(Some(far), now, now),
            now + Duration::days(30)
        );
    }

    #[test]
    fn a_device_is_expired_from_its_expiry_instant() {
        let now = Utc::now();
        let device = DeviceSession::new(
            Uuid::new_v4(),
            "h".to_owned(),
            now + Duration::hours(1),
            now,
        );

        assert!(!device.is_expired(now));
        assert!(device.is_expired(now + Duration::hours(1)));
    }

    #[test]
    fn a_device_belongs_to_its_realm() {
        let realm = Uuid::new_v4();
        let device = DeviceSession::new(realm, "h".to_owned(), Utc::now(), Utc::now());

        assert_eq!(Uuid::from(device.realm_id()), realm);
    }
}
