use std::sync::Arc;

use chrono::{DateTime, Utc};
use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::realm::scope::{Scoped, Unscoped};
use ferriskey_domain::session::entities::{
    SessionError, SessionFilter, SessionSortField, UserSession,
};
use ferriskey_domain::session::ports::UserSessionRepository;
use ferriskey_domain::user::entities::User;
use uuid::Uuid;

use super::state::TokenState;
use crate::infrastructure::common::Retention;

#[derive(Debug, Clone)]
pub struct CachedUserSessionRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> CachedUserSessionRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: UserSessionRepository> UserSessionRepository for CachedUserSessionRepository<R> {
    async fn create(&self, session: &UserSession) -> Result<(), SessionError> {
        self.inner.create(session).await
    }

    async fn find_by_user_id(&self, user_id: &Uuid) -> Result<Unscoped<UserSession>, SessionError> {
        self.inner.find_by_user_id(user_id).await
    }

    async fn find_all_by_user_and_realm(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Vec<UserSession>, SessionError> {
        self.inner
            .find_all_by_user_and_realm(user_id, realm_id)
            .await
    }

    async fn list(
        &self,
        user: &Scoped<User>,
        request: &PageRequest<SessionFilter, SessionSortField>,
    ) -> Result<Page<UserSession>, SessionError> {
        self.inner.list(user, request).await
    }

    async fn find_by_id(
        &self,
        session_id: Uuid,
    ) -> Result<Option<Unscoped<UserSession>>, SessionError> {
        if !self.state.synced() {
            return self.inner.find_by_id(session_id).await;
        }

        let read_at = self.state.generation();
        self.state
            .sessions()
            .load_while(
                &session_id,
                || self.state.generation() == read_at,
                || async {
                    let session = self.inner.find_by_id(session_id).await?;
                    let retention = if session.is_some() && self.state.generation() == read_at {
                        Retention::Keep
                    } else {
                        Retention::Own
                    };

                    Ok((session, retention))
                },
            )
            .await
    }

    async fn find_by_sso_token_hash(
        &self,
        sso_token_hash: &str,
    ) -> Result<Option<Unscoped<UserSession>>, SessionError> {
        self.inner.find_by_sso_token_hash(sso_token_hash).await
    }

    async fn delete(&self, session: &Scoped<UserSession>) -> Result<(), SessionError> {
        let deleted = self.inner.delete(session).await;
        self.state.changed();
        deleted
    }

    async fn delete_all_by_user(&self, user_id: Uuid, realm_id: Uuid) -> Result<u64, SessionError> {
        let deleted = self.inner.delete_all_by_user(user_id, realm_id).await;
        self.state.changed();
        deleted
    }

    async fn delete_expired(&self, before: DateTime<Utc>, limit: u64) -> Result<u64, SessionError> {
        self.inner.delete_expired(before, limit).await
    }

    async fn update_last_seen(&self, session: &Scoped<UserSession>) -> Result<(), SessionError> {
        self.inner.update_last_seen(session).await
    }

    async fn set_sso_token_hash(
        &self,
        session: &Scoped<UserSession>,
        sso_token_hash: &str,
    ) -> Result<(), SessionError> {
        self.inner.set_sso_token_hash(session, sso_token_hash).await
    }

    async fn reauthenticate(&self, session: &Scoped<UserSession>) -> Result<(), SessionError> {
        let reauthenticated = self.inner.reauthenticate(session).await;
        self.state.changed();
        reauthenticated
    }

    async fn clear_sso_token_hash(
        &self,
        session: &Scoped<UserSession>,
    ) -> Result<(), SessionError> {
        let cleared = self.inner.clear_sso_token_hash(session).await;
        self.state.changed();
        cleared
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chrono::{Duration as Span, Utc};
    use ferriskey_domain::realm::scope::{RealmScope, Unscoped};
    use ferriskey_domain::realm::{Realm, RealmId};
    use ferriskey_domain::session::entities::UserSession;
    use ferriskey_domain::session::ports::{MockUserSessionRepository, UserSessionRepository};
    use std::sync::Arc;
    use uuid::Uuid;

    use super::CachedUserSessionRepository;
    use crate::infrastructure::token_state::TokenState;

    const TTL: Duration = Duration::from_secs(60);

    fn session(realm_id: Uuid) -> UserSession {
        UserSession {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            realm_id,
            user_agent: None,
            ip_address: None,
            created_at: Utc::now(),
            expires_at: Utc::now() + Span::hours(1),
            last_seen_at: None,
            soft_expiry_duration: None,
            sso_token_hash: None,
            persistent: false,
            authenticated_at: Utc::now(),
        }
    }

    fn cached(
        inner: MockUserSessionRepository,
    ) -> CachedUserSessionRepository<MockUserSessionRepository> {
        let (state, _feed) = TokenState::new(TTL);
        state.set_synced(true);

        CachedUserSessionRepository::new(Arc::new(inner), state)
    }

    fn finding(stored: Option<UserSession>, calls: usize) -> MockUserSessionRepository {
        let mut inner = MockUserSessionRepository::new();
        inner.expect_find_by_id().times(calls).returning(move |_| {
            let stored = stored.clone();
            Box::pin(async move { Ok(stored.map(Unscoped::new)) })
        });
        inner
    }

    fn scoped(session: UserSession) -> ferriskey_domain::realm::scope::Scoped<UserSession> {
        let mut realm = Realm::new("sessions".to_string());
        realm.id = RealmId::new(session.realm_id);
        Unscoped::new(session)
            .in_realm(&RealmScope::from_realm(realm))
            .expect("a session of its own realm")
    }

    #[tokio::test]
    async fn a_live_session_is_served_from_memory() {
        let live = session(Uuid::new_v4());
        let repository = cached(finding(Some(live.clone()), 1));

        let first = repository.find_by_id(live.id).await.expect("first");
        let second = repository.find_by_id(live.id).await.expect("second");

        assert_eq!(first.map(|s| s.across_realms().id), Some(live.id));
        assert_eq!(second.map(|s| s.across_realms().id), Some(live.id));
    }

    #[tokio::test]
    async fn a_missing_session_is_asked_about_every_time() {
        let repository = cached(finding(None, 2));
        let id = Uuid::new_v4();

        assert!(repository.find_by_id(id).await.expect("first").is_none());
        assert!(repository.find_by_id(id).await.expect("second").is_none());
    }

    #[tokio::test]
    async fn deleting_a_session_makes_the_next_read_go_to_the_store() {
        let live = session(Uuid::new_v4());
        let mut inner = finding(Some(live.clone()), 2);
        inner
            .expect_delete()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));
        let repository = cached(inner);
        repository.find_by_id(live.id).await.expect("warm");

        repository
            .delete(&scoped(live.clone()))
            .await
            .expect("delete");

        repository.find_by_id(live.id).await.expect("read again");
    }

    #[tokio::test]
    async fn ending_every_session_of_a_user_empties_the_cache() {
        let live = session(Uuid::new_v4());
        let mut inner = finding(Some(live.clone()), 2);
        inner
            .expect_delete_all_by_user()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(1) }));
        let repository = cached(inner);
        repository.find_by_id(live.id).await.expect("warm");

        repository
            .delete_all_by_user(live.user_id, live.realm_id)
            .await
            .expect("delete all");

        repository.find_by_id(live.id).await.expect("read again");
    }

    #[tokio::test]
    async fn a_read_overtaken_by_a_deletion_is_not_kept() {
        let live = session(Uuid::new_v4());
        let (state, _feed) = TokenState::new(TTL);
        state.set_synced(true);
        let racing = Arc::clone(&state);
        let stored = live.clone();
        let mut inner = MockUserSessionRepository::new();
        inner.expect_find_by_id().times(2).returning(move |_| {
            racing.changed();
            let stored = stored.clone();
            Box::pin(async move { Ok(Some(Unscoped::new(stored))) })
        });
        let repository = CachedUserSessionRepository::new(Arc::new(inner), state);

        repository.find_by_id(live.id).await.expect("first");
        repository.find_by_id(live.id).await.expect("second");
    }

    #[tokio::test]
    async fn cutting_the_sso_secret_makes_the_next_read_go_to_the_store() {
        let live = session(Uuid::new_v4());
        let mut inner = finding(Some(live.clone()), 2);
        inner
            .expect_clear_sso_token_hash()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));
        let repository = cached(inner);
        repository.find_by_id(live.id).await.expect("warm");

        repository
            .clear_sso_token_hash(&scoped(live.clone()))
            .await
            .expect("clear");

        repository.find_by_id(live.id).await.expect("read again");
    }

    #[tokio::test]
    async fn reauthenticating_a_session_makes_the_next_read_go_to_the_store() {
        let live = session(Uuid::new_v4());
        let mut inner = finding(Some(live.clone()), 2);
        inner
            .expect_reauthenticate()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));
        let repository = cached(inner);
        repository.find_by_id(live.id).await.expect("warm");

        repository
            .reauthenticate(&scoped(live.clone()))
            .await
            .expect("reauthenticate");

        repository.find_by_id(live.id).await.expect("read again");
    }

    #[tokio::test]
    async fn while_the_state_is_not_in_sync_every_read_goes_to_the_store() {
        let live = session(Uuid::new_v4());
        let (state, _feed) = TokenState::new(TTL);
        let repository =
            CachedUserSessionRepository::new(Arc::new(finding(Some(live.clone()), 3)), state);

        for _ in 0..3 {
            repository.find_by_id(live.id).await.expect("read");
        }
    }
}
