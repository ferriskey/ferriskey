use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::entities::JwtKeyPair;

pub const SIGNING_ALGORITHM: &str = "RS256";

/// Matches the leeway `jsonwebtoken` grants on `exp`, so a token accepted at the
/// edge of its lifetime is never refused because its key retired a second earlier.
pub const VERIFICATION_LEEWAY: Duration = Duration::seconds(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SigningKeyStatus {
    Active,
    Passive,
    Retired,
    Revoked,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SigningKeyLifecycle {
    pub rotated_at: Option<DateTime<Utc>>,
    pub retire_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl SigningKeyLifecycle {
    pub fn status_at(&self, now: DateTime<Utc>) -> SigningKeyStatus {
        if self.revoked_at.is_some() {
            return SigningKeyStatus::Revoked;
        }

        match self.retire_at {
            None => SigningKeyStatus::Active,
            Some(retire_at) if now < retire_at => SigningKeyStatus::Passive,
            Some(_) => SigningKeyStatus::Retired,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SigningKey {
    pub kid: Uuid,
    pub realm_id: Uuid,
    pub algorithm: String,
    pub status: SigningKeyStatus,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub retire_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl SigningKey {
    pub fn new(
        kid: Uuid,
        realm_id: Uuid,
        created_at: DateTime<Utc>,
        lifecycle: SigningKeyLifecycle,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            kid,
            realm_id,
            algorithm: SIGNING_ALGORITHM.to_string(),
            status: lifecycle.status_at(now),
            created_at,
            rotated_at: lifecycle.rotated_at,
            retire_at: lifecycle.retire_at,
            revoked_at: lifecycle.revoked_at,
        }
    }
}

/// How long a rotated key keeps verifying: the longest lifetime a token signed
/// by it can have, plus the verification leeway.
pub fn retirement_grace(lifetimes_in_seconds: impl IntoIterator<Item = i64>) -> Duration {
    let longest = lifetimes_in_seconds.into_iter().max().unwrap_or(0).max(0);

    Duration::seconds(longest) + VERIFICATION_LEEWAY
}

#[derive(Clone)]
pub struct PassiveKey {
    pub key: JwtKeyPair,
    pub retire_at: DateTime<Utc>,
}

/// The keys of a realm that are published: the active one, which signs, and the
/// passive ones, which only verify until they retire.
#[derive(Clone)]
pub struct RealmKeySet {
    active: JwtKeyPair,
    passive: Vec<PassiveKey>,
}

impl RealmKeySet {
    pub fn new(active: JwtKeyPair, passive: Vec<PassiveKey>) -> Self {
        Self { active, passive }
    }

    pub fn active(&self) -> &JwtKeyPair {
        &self.active
    }

    pub fn all(&self) -> impl Iterator<Item = &JwtKeyPair> {
        std::iter::once(&self.active).chain(self.passive.iter().map(|passive| &passive.key))
    }

    pub fn published_at(&self, now: DateTime<Utc>) -> impl Iterator<Item = &JwtKeyPair> {
        std::iter::once(&self.active).chain(
            self.passive
                .iter()
                .filter(move |passive| now < passive.retire_at)
                .map(|passive| &passive.key),
        )
    }

    /// The key that verifies a token carrying `kid`. A token without a `kid`
    /// predates rotation and can only have been signed by the active key.
    pub fn verification_key(&self, kid: Option<&str>, now: DateTime<Utc>) -> Option<&JwtKeyPair> {
        let Some(kid) = kid else {
            return Some(&self.active);
        };
        let kid = Uuid::parse_str(kid).ok()?;

        self.published_at(now).find(|key| key.id == kid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key_pair(id: Uuid) -> JwtKeyPair {
        let (private_key, public_key) = JwtKeyPair::generate().expect("generate key");
        let certificate =
            JwtKeyPair::self_signed_certificate(&private_key, "tenant").expect("certificate");
        JwtKeyPair::from_pem(&private_key, &public_key, &certificate, Uuid::nil(), id)
            .expect("key pair")
    }

    fn at(seconds: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_800_000_000 + seconds, 0).expect("timestamp")
    }

    #[test]
    fn a_key_never_rotated_nor_revoked_is_active() {
        assert_eq!(
            SigningKeyLifecycle::default().status_at(at(0)),
            SigningKeyStatus::Active
        );
    }

    #[test]
    fn a_rotated_key_is_passive_until_it_retires() {
        let lifecycle = SigningKeyLifecycle {
            rotated_at: Some(at(0)),
            retire_at: Some(at(100)),
            revoked_at: None,
        };

        assert_eq!(lifecycle.status_at(at(99)), SigningKeyStatus::Passive);
        assert_eq!(lifecycle.status_at(at(100)), SigningKeyStatus::Retired);
    }

    #[test]
    fn revocation_wins_over_every_other_state() {
        let active = SigningKeyLifecycle {
            revoked_at: Some(at(0)),
            ..SigningKeyLifecycle::default()
        };
        let passive = SigningKeyLifecycle {
            rotated_at: Some(at(0)),
            retire_at: Some(at(100)),
            revoked_at: Some(at(10)),
        };

        assert_eq!(active.status_at(at(1)), SigningKeyStatus::Revoked);
        assert_eq!(passive.status_at(at(50)), SigningKeyStatus::Revoked);
    }

    #[test]
    fn the_grace_covers_the_longest_lifetime_plus_the_leeway() {
        assert_eq!(
            retirement_grace([300, 86_400, 300, 300, 172_800]),
            Duration::seconds(172_800 + 60)
        );
    }

    #[test]
    fn the_grace_is_never_below_the_leeway() {
        assert_eq!(retirement_grace([]), VERIFICATION_LEEWAY);
        assert_eq!(retirement_grace([-5]), VERIFICATION_LEEWAY);
    }

    #[test]
    fn the_active_key_is_published_first_and_retired_keys_are_dropped() {
        let active = Uuid::new_v4();
        let passive = Uuid::new_v4();
        let retired = Uuid::new_v4();
        let set = RealmKeySet::new(
            key_pair(active),
            vec![
                PassiveKey {
                    key: key_pair(passive),
                    retire_at: at(100),
                },
                PassiveKey {
                    key: key_pair(retired),
                    retire_at: at(10),
                },
            ],
        );

        let published: Vec<Uuid> = set.published_at(at(50)).map(|key| key.id).collect();

        assert_eq!(published, vec![active, passive]);
    }

    #[test]
    fn a_token_is_verified_by_the_key_its_kid_names() {
        let active = Uuid::new_v4();
        let passive = Uuid::new_v4();
        let set = RealmKeySet::new(
            key_pair(active),
            vec![PassiveKey {
                key: key_pair(passive),
                retire_at: at(100),
            }],
        );

        let found = set.verification_key(Some(&passive.to_string()), at(50));
        let after_retirement = set.verification_key(Some(&passive.to_string()), at(100));
        let unknown = set.verification_key(Some(&Uuid::new_v4().to_string()), at(50));
        let without_kid = set.verification_key(None, at(50));

        assert_eq!(found.map(|key| key.id), Some(passive));
        assert!(after_retirement.is_none());
        assert!(unknown.is_none());
        assert_eq!(without_kid.map(|key| key.id), Some(active));
    }
}
