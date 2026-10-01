use std::sync::Arc;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::authentication::backchannel_logout::{BackchannelLogoutNotifier, EndedSession};
use crate::domain::realm::entities::{Scoped, Unscoped};
use crate::domain::session::{
    entities::{SessionError, UserSession},
    ports::UserSessionRepository,
};

/// Wraps the session store so that every session deleted through it, by
/// logout, revocation, password change or user disable, is reported to the
/// back-channel logout notifier. Ending a session anywhere goes through
/// `delete` or `delete_all_by_user`, so no caller can forget to notify.
/// Purging expired sessions does not notify: an expired session already
/// ended on its own.
#[derive(Debug, Clone)]
pub struct NotifyingUserSessionRepository<R, N>
where
    R: UserSessionRepository,
    N: BackchannelLogoutNotifier,
{
    inner: Arc<R>,
    notifier: N,
}

impl<R, N> NotifyingUserSessionRepository<R, N>
where
    R: UserSessionRepository,
    N: BackchannelLogoutNotifier,
{
    pub fn new(inner: Arc<R>, notifier: N) -> Self {
        Self { inner, notifier }
    }
}

fn ended(session: &UserSession) -> EndedSession {
    EndedSession {
        realm_id: session.realm_id.into(),
        session_id: session.id,
        user_id: session.user_id,
    }
}

impl<R, N> UserSessionRepository for NotifyingUserSessionRepository<R, N>
where
    R: UserSessionRepository,
    N: BackchannelLogoutNotifier,
{
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

    async fn find_by_id(
        &self,
        session_id: Uuid,
    ) -> Result<Option<Unscoped<UserSession>>, SessionError> {
        self.inner.find_by_id(session_id).await
    }

    async fn find_by_sso_token_hash(
        &self,
        sso_token_hash: &str,
    ) -> Result<Option<Unscoped<UserSession>>, SessionError> {
        self.inner.find_by_sso_token_hash(sso_token_hash).await
    }

    async fn delete(&self, session: &Scoped<UserSession>) -> Result<(), SessionError> {
        self.inner.delete(session).await?;
        self.notifier.sessions_ended(vec![ended(session.get())]);
        Ok(())
    }

    async fn delete_all_by_user(&self, user_id: Uuid, realm_id: Uuid) -> Result<u64, SessionError> {
        let live: Vec<EndedSession> = self
            .inner
            .find_all_by_user_and_realm(user_id, realm_id)
            .await
            .unwrap_or_default()
            .iter()
            .filter(|session| !session.is_expired())
            .map(ended)
            .collect();

        let deleted = self.inner.delete_all_by_user(user_id, realm_id).await?;
        self.notifier.sessions_ended(live);
        Ok(deleted)
    }

    async fn delete_expired_for_user(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<u64, SessionError> {
        self.inner
            .delete_expired_for_user(user_id, realm_id, now)
            .await
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
        self.inner.reauthenticate(session).await
    }

    async fn clear_sso_token_hash(
        &self,
        session: &Scoped<UserSession>,
    ) -> Result<(), SessionError> {
        self.inner.clear_sso_token_hash(session).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use chrono::Duration;

    use super::*;
    use crate::domain::common::services::tests::create_test_realm;
    use crate::domain::realm::entities::RealmScope;
    use crate::domain::session::ports::MockUserSessionRepository;

    #[derive(Default, Clone)]
    struct RecordingNotifier(Arc<Mutex<Vec<EndedSession>>>);

    impl BackchannelLogoutNotifier for RecordingNotifier {
        fn sessions_ended(&self, sessions: Vec<EndedSession>) {
            self.0.lock().expect("notified lock").extend(sessions);
        }
    }

    fn user_session(user_id: Uuid, realm_id: Uuid, expires_in: Duration) -> UserSession {
        let mut session = UserSession::new(user_id, realm_id, None, None, Duration::hours(1), None);
        session.expires_at = Utc::now() + expires_in;
        session
    }

    fn scoped(session: UserSession) -> Scoped<UserSession> {
        let mut realm = create_test_realm();
        realm.id = session.realm_id.into();
        Unscoped::new(session)
            .in_realm(&RealmScope::from_realm(realm))
            .expect("the session belongs to its realm")
    }

    #[tokio::test]
    async fn deleting_a_session_reports_it() {
        let session = user_session(Uuid::new_v4(), Uuid::new_v4(), Duration::hours(1));
        let mut inner = MockUserSessionRepository::new();
        inner
            .expect_delete()
            .returning(|_| Box::pin(async { Ok(()) }));
        let notifier = RecordingNotifier::default();
        let repository = NotifyingUserSessionRepository::new(Arc::new(inner), notifier.clone());

        repository
            .delete(&scoped(session.clone()))
            .await
            .expect("delete succeeds");

        let notified = notifier.0.lock().expect("notified lock");
        assert_eq!(notified.len(), 1);
        assert_eq!(notified[0].session_id, session.id);
        assert_eq!(notified[0].user_id, session.user_id);
    }

    #[tokio::test]
    async fn a_failed_delete_reports_nothing() {
        let session = user_session(Uuid::new_v4(), Uuid::new_v4(), Duration::hours(1));
        let mut inner = MockUserSessionRepository::new();
        inner
            .expect_delete()
            .returning(|_| Box::pin(async { Err(SessionError::NotFound) }));
        let notifier = RecordingNotifier::default();
        let repository = NotifyingUserSessionRepository::new(Arc::new(inner), notifier.clone());

        let _ = repository.delete(&scoped(session)).await;

        assert!(notifier.0.lock().expect("notified lock").is_empty());
    }

    #[tokio::test]
    async fn deleting_every_session_of_a_user_reports_the_live_ones() {
        let (user_id, realm_id) = (Uuid::new_v4(), Uuid::new_v4());
        let live = user_session(user_id, realm_id, Duration::hours(1));
        let expired = user_session(user_id, realm_id, -Duration::hours(1));
        let listed = vec![live.clone(), expired];
        let mut inner = MockUserSessionRepository::new();
        inner
            .expect_find_all_by_user_and_realm()
            .returning(move |_, _| {
                let listed = listed.clone();
                Box::pin(async move { Ok(listed) })
            });
        inner
            .expect_delete_all_by_user()
            .returning(|_, _| Box::pin(async { Ok(2) }));
        let notifier = RecordingNotifier::default();
        let repository = NotifyingUserSessionRepository::new(Arc::new(inner), notifier.clone());

        let deleted = repository
            .delete_all_by_user(user_id, realm_id)
            .await
            .expect("delete succeeds");

        assert_eq!(deleted, 2);
        let notified = notifier.0.lock().expect("notified lock");
        assert_eq!(
            notified.len(),
            1,
            "an expired session already ended on its own"
        );
        assert_eq!(notified[0].session_id, live.id);
    }

    #[tokio::test]
    async fn purging_expired_sessions_reports_nothing() {
        let mut inner = MockUserSessionRepository::new();
        inner
            .expect_delete_expired_for_user()
            .returning(|_, _, _| Box::pin(async { Ok(3) }));
        let notifier = RecordingNotifier::default();
        let repository = NotifyingUserSessionRepository::new(Arc::new(inner), notifier.clone());

        repository
            .delete_expired_for_user(Uuid::new_v4(), Uuid::new_v4(), Utc::now())
            .await
            .expect("purge succeeds");

        assert!(notifier.0.lock().expect("notified lock").is_empty());
    }
}
