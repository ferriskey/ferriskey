use std::sync::Arc;

use chrono::{DateTime, Utc};
use ferriskey_domain::realm::RealmId;
use ferriskey_domain::realm::scope::Unscoped;
use ferriskey_security::SecurityError;
use ferriskey_security::jwt::entities::AccessToken;
use ferriskey_security::jwt::ports::AccessTokenRepository;
use uuid::Uuid;

use super::state::TokenState;
use crate::infrastructure::common::Retention;

#[derive(Debug, Clone)]
pub struct CachedAccessTokenRepository<R> {
    inner: R,
    state: Arc<TokenState>,
}

impl<R> CachedAccessTokenRepository<R> {
    pub fn new(inner: R, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: AccessTokenRepository> AccessTokenRepository for CachedAccessTokenRepository<R> {
    async fn record_session_participant(
        &self,
        realm_id: RealmId,
        session_id: Uuid,
        client_id: String,
        issuer: String,
    ) -> Result<(), SecurityError> {
        self.inner
            .record_session_participant(realm_id, session_id, client_id, issuer)
            .await
    }

    async fn get_by_token_hash(
        &self,
        token_hash: String,
    ) -> Result<Option<Unscoped<AccessToken>>, SecurityError> {
        self.inner.get_by_token_hash(token_hash).await
    }

    async fn is_revoked(
        &self,
        token_hash: String,
        user_id: Uuid,
        session_id: Option<Uuid>,
        issued_at: i64,
    ) -> Result<bool, SecurityError> {
        if !self.state.synced() {
            return self
                .inner
                .is_revoked(token_hash, user_id, session_id, issued_at)
                .await;
        }

        let read_at = self.state.generation();
        self.state
            .unrevoked()
            .load_while(
                token_hash.as_str(),
                || self.state.generation() == read_at,
                || async {
                    let revoked = self
                        .inner
                        .is_revoked(token_hash.clone(), user_id, session_id, issued_at)
                        .await?;
                    let retention = match (revoked, self.state.generation() == read_at) {
                        (true, _) => Retention::Share,
                        (false, true) => Retention::Keep,
                        (false, false) => Retention::Own,
                    };

                    Ok((revoked, retention))
                },
            )
            .await
    }

    async fn revoke_by_token_hash(
        &self,
        token_hash: String,
        jti: Option<Uuid>,
        user_id: Uuid,
        realm_id: RealmId,
        expires_at: Option<DateTime<Utc>>,
        claims: serde_json::Value,
    ) -> Result<(), SecurityError> {
        let revoked = self
            .inner
            .revoke_by_token_hash(token_hash, jti, user_id, realm_id, expires_at, claims)
            .await;
        self.state.changed();
        revoked
    }

    async fn revoke_by_session_id(&self, session_id: Uuid) -> Result<u64, SecurityError> {
        let revoked = self.inner.revoke_by_session_id(session_id).await;
        self.state.changed();
        revoked
    }

    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<u64, SecurityError> {
        let revoked = self.inner.revoke_all_for_user(user_id).await;
        self.state.changed();
        revoked
    }

    async fn delete_expired(
        &self,
        before: DateTime<Utc>,
        limit: u64,
    ) -> Result<u64, SecurityError> {
        self.inner.delete_expired(before, limit).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use ferriskey_security::jwt::ports::{AccessTokenRepository, MockAccessTokenRepository};
    use uuid::Uuid;

    use super::CachedAccessTokenRepository;
    use crate::infrastructure::token_state::TokenState;

    const TTL: Duration = Duration::from_secs(60);

    fn cached(
        inner: MockAccessTokenRepository,
    ) -> CachedAccessTokenRepository<MockAccessTokenRepository> {
        let (state, _feed) = TokenState::new(TTL);
        state.set_synced(true);

        CachedAccessTokenRepository::new(inner, state)
    }

    fn answering(revoked: bool, calls: usize) -> MockAccessTokenRepository {
        let mut inner = MockAccessTokenRepository::new();
        inner
            .expect_is_revoked()
            .times(calls)
            .returning(move |_, _, _, _| Box::pin(async move { Ok(revoked) }));
        inner
    }

    #[tokio::test]
    async fn a_token_found_not_revoked_is_not_asked_about_again() {
        let repository = cached(answering(false, 1));
        let user = Uuid::new_v4();

        let first = repository
            .is_revoked("hash".to_string(), user, None, 10)
            .await;
        let second = repository
            .is_revoked("hash".to_string(), user, None, 10)
            .await;

        assert_eq!((first.ok(), second.ok()), (Some(false), Some(false)));
    }

    #[tokio::test]
    async fn a_revoked_answer_is_never_kept() {
        let repository = cached(answering(true, 2));
        let user = Uuid::new_v4();

        let first = repository
            .is_revoked("hash".to_string(), user, None, 10)
            .await;
        let second = repository
            .is_revoked("hash".to_string(), user, None, 10)
            .await;

        assert_eq!((first.ok(), second.ok()), (Some(true), Some(true)));
    }

    #[tokio::test]
    async fn revoking_a_token_makes_the_next_check_read_the_store() {
        let mut inner = answering(false, 2);
        inner
            .expect_revoke_by_token_hash()
            .times(1)
            .returning(|_, _, _, _, _, _| Box::pin(async { Ok(()) }));
        let repository = cached(inner);
        let user = Uuid::new_v4();
        repository
            .is_revoked("hash".to_string(), user, None, 10)
            .await
            .expect("check");

        repository
            .revoke_by_token_hash(
                "hash".to_string(),
                None,
                user,
                ferriskey_domain::realm::RealmId::new(Uuid::new_v4()),
                None,
                serde_json::json!({}),
            )
            .await
            .expect("revoke");

        repository
            .is_revoked("hash".to_string(), user, None, 10)
            .await
            .expect("check again");
    }

    #[tokio::test]
    async fn revoking_every_token_of_a_user_empties_the_cache_even_when_it_fails() {
        let mut inner = answering(false, 2);
        inner.expect_revoke_all_for_user().times(1).returning(|_| {
            Box::pin(async {
                Err(ferriskey_security::SecurityError::GenerationError(
                    "down".to_string(),
                ))
            })
        });
        let repository = cached(inner);
        let user = Uuid::new_v4();
        repository
            .is_revoked("hash".to_string(), user, None, 10)
            .await
            .expect("check");

        assert!(repository.revoke_all_for_user(user).await.is_err());

        repository
            .is_revoked("hash".to_string(), user, None, 10)
            .await
            .expect("check again");
    }

    #[tokio::test]
    async fn an_answer_overtaken_by_a_revocation_is_not_kept() {
        let (state, _feed) = TokenState::new(TTL);
        state.set_synced(true);
        let racing = Arc::clone(&state);
        let mut inner = MockAccessTokenRepository::new();
        inner
            .expect_is_revoked()
            .times(2)
            .returning(move |_, _, _, _| {
                racing.changed();
                Box::pin(async { Ok(false) })
            });
        let repository = CachedAccessTokenRepository::new(inner, state);
        let user = Uuid::new_v4();

        repository
            .is_revoked("hash".to_string(), user, None, 10)
            .await
            .expect("first");
        repository
            .is_revoked("hash".to_string(), user, None, 10)
            .await
            .expect("second");
    }

    #[tokio::test]
    async fn while_the_state_is_not_in_sync_every_check_reads_the_store() {
        let (state, _feed) = TokenState::new(TTL);
        let repository = CachedAccessTokenRepository::new(answering(false, 3), state);
        let user = Uuid::new_v4();

        for _ in 0..3 {
            repository
                .is_revoked("hash".to_string(), user, None, 10)
                .await
                .expect("check");
        }
    }
}
