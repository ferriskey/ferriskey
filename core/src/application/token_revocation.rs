use std::sync::Arc;

use ferriskey_domain::realm::scope::Scoped;
use ferriskey_domain::session::entities::UserSession;
use ferriskey_domain::session::ports::{TokenRevocationPort, UserSessionRepository};
use ferriskey_security::jwt::ports::{AccessTokenRepository, RefreshTokenRepository};
use tracing::warn;
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;

#[derive(Clone, Debug)]
pub struct TokenRevocationAdapter<A, R, S>
where
    A: AccessTokenRepository,
    R: RefreshTokenRepository,
    S: UserSessionRepository,
{
    access_token_repository: Arc<A>,
    refresh_token_repository: Arc<R>,
    session_repository: Arc<S>,
}

impl<A, R, S> TokenRevocationAdapter<A, R, S>
where
    A: AccessTokenRepository,
    R: RefreshTokenRepository,
    S: UserSessionRepository,
{
    pub fn new(
        access_token_repository: Arc<A>,
        refresh_token_repository: Arc<R>,
        session_repository: Arc<S>,
    ) -> Self {
        Self {
            access_token_repository,
            refresh_token_repository,
            session_repository,
        }
    }
}

impl<A, R, S> TokenRevocationPort for TokenRevocationAdapter<A, R, S>
where
    A: AccessTokenRepository,
    R: RefreshTokenRepository,
    S: UserSessionRepository,
{
    async fn revoke_session_tokens(&self, session: &Scoped<UserSession>) -> Result<(), CoreError> {
        let session_id = session.get().id;

        let refresh_revoked = self
            .refresh_token_repository
            .revoke_by_session_id(session_id)
            .await
            .map_err(|e| {
                warn!("failed to revoke refresh tokens for session {session_id}: {e:?}");
                CoreError::InternalServerError
            })?;

        let access_revoked = self
            .access_token_repository
            .revoke_by_session_id(session_id)
            .await
            .map_err(|e| {
                warn!("failed to revoke access tokens for session {session_id}: {e:?}");
                CoreError::InternalServerError
            })?;

        tracing::debug!(
            "session {session_id} revoked: {refresh_revoked} refresh token(s), {access_revoked} access token(s)"
        );

        Ok(())
    }

    async fn revoke_all_user_access(&self, user_id: Uuid, realm_id: Uuid) -> Result<(), CoreError> {
        // Sessions go first: while one survives, its SSO cookie can mint fresh
        // tokens at `/auth` right after the revocation below. Tokens are
        // revoked by user, not through the session rows, so deleting the rows
        // first loses nothing, and a failure here still revokes the tokens
        // before reporting it.
        let sessions_deleted = self
            .session_repository
            .delete_all_by_user(user_id, realm_id)
            .await
            .map_err(|e| {
                warn!("failed to delete the sessions of user {user_id} in realm {realm_id}: {e:?}");
                CoreError::InternalServerError
            });

        let refresh_revoked = self
            .refresh_token_repository
            .revoke_all_for_user(user_id)
            .await
            .map_err(|e| {
                warn!("failed to revoke refresh tokens for user {user_id}: {e:?}");
                CoreError::InternalServerError
            })?;

        let access_revoked = self
            .access_token_repository
            .revoke_all_for_user(user_id)
            .await
            .map_err(|e| {
                warn!("failed to revoke access tokens for user {user_id}: {e:?}");
                CoreError::InternalServerError
            })?;

        let sessions_deleted = sessions_deleted?;

        tracing::debug!(
            "user {user_id} access revoked: {sessions_deleted} session(s), {refresh_revoked} refresh token(s), {access_revoked} access token(s)"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use ferriskey_domain::realm::scope::{RealmScope, Unscoped};
    use ferriskey_domain::realm::{Realm, RealmId};
    use ferriskey_domain::session::entities::SessionError;
    use ferriskey_domain::session::ports::MockUserSessionRepository;
    use ferriskey_security::jwt::ports::{MockAccessTokenRepository, MockRefreshTokenRepository};

    fn make_session(user_id: Uuid, realm_id: Uuid) -> UserSession {
        UserSession {
            id: Uuid::new_v4(),
            user_id,
            realm_id,
            user_agent: None,
            ip_address: None,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + Duration::hours(1),
            last_seen_at: None,
            soft_expiry_duration: None,
            sso_token_hash: None,
            persistent: false,
            authenticated_at: chrono::Utc::now(),
        }
    }

    fn scoped_session(user_id: Uuid, realm_id: Uuid) -> Scoped<UserSession> {
        let mut realm = Realm::new("token-revocation".to_string());
        realm.id = RealmId::new(realm_id);

        Unscoped::new(make_session(user_id, realm_id))
            .in_realm(&RealmScope::from_realm(realm))
            .expect("a session of the scoped realm must be accepted")
    }

    #[tokio::test]
    async fn revoke_session_tokens_hits_both_token_stores() {
        let session = scoped_session(Uuid::new_v4(), Uuid::new_v4());
        let session_id = session.get().id;

        let mut access = MockAccessTokenRepository::new();
        access
            .expect_revoke_by_session_id()
            .with(mockall::predicate::eq(session_id))
            .times(1)
            .return_once(|_| Box::pin(async { Ok(1) }));

        let mut refresh = MockRefreshTokenRepository::new();
        refresh
            .expect_revoke_by_session_id()
            .with(mockall::predicate::eq(session_id))
            .times(1)
            .return_once(|_| Box::pin(async { Ok(1) }));

        let adapter = TokenRevocationAdapter::new(
            Arc::new(access),
            Arc::new(refresh),
            Arc::new(MockUserSessionRepository::new()),
        );

        assert!(adapter.revoke_session_tokens(&session).await.is_ok());
    }

    #[tokio::test]
    async fn revoke_all_user_access_drops_sessions_before_revoking_tokens() {
        let user_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));

        let mut session_repo = MockUserSessionRepository::new();
        let log = calls.clone();
        session_repo
            .expect_delete_all_by_user()
            .with(
                mockall::predicate::eq(user_id),
                mockall::predicate::eq(realm_id),
            )
            .times(1)
            .return_once(move |_, _| {
                log.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .push("sessions");
                Box::pin(async { Ok(2) })
            });

        let mut refresh = MockRefreshTokenRepository::new();
        let log = calls.clone();
        refresh
            .expect_revoke_all_for_user()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .return_once(move |_| {
                log.lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .push("refresh");
                Box::pin(async { Ok(2) })
            });

        let mut access = MockAccessTokenRepository::new();
        let log = calls.clone();
        access
            .expect_revoke_all_for_user()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .return_once(move |_| {
                log.lock().unwrap_or_else(|p| p.into_inner()).push("access");
                Box::pin(async { Ok(3) })
            });

        let adapter = TokenRevocationAdapter::new(
            Arc::new(access),
            Arc::new(refresh),
            Arc::new(session_repo),
        );

        assert!(
            adapter
                .revoke_all_user_access(user_id, realm_id)
                .await
                .is_ok()
        );
        assert_eq!(
            *calls.lock().unwrap_or_else(|p| p.into_inner()),
            ["sessions", "refresh", "access"],
            "sessions must be gone before any token is revoked"
        );
    }

    #[tokio::test]
    async fn revoke_all_user_access_still_revokes_tokens_when_sessions_survive() {
        let user_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();

        let mut session_repo = MockUserSessionRepository::new();
        session_repo
            .expect_delete_all_by_user()
            .times(1)
            .return_once(|_, _| Box::pin(async { Err(SessionError::DeleteError) }));

        let mut refresh = MockRefreshTokenRepository::new();
        refresh
            .expect_revoke_all_for_user()
            .times(1)
            .return_once(|_| Box::pin(async { Ok(1) }));

        let mut access = MockAccessTokenRepository::new();
        access
            .expect_revoke_all_for_user()
            .times(1)
            .return_once(|_| Box::pin(async { Ok(1) }));

        let adapter = TokenRevocationAdapter::new(
            Arc::new(access),
            Arc::new(refresh),
            Arc::new(session_repo),
        );

        assert!(
            adapter
                .revoke_all_user_access(user_id, realm_id)
                .await
                .is_err(),
            "a surviving session keeps its SSO cookie alive, so it must not report success"
        );
    }

    #[tokio::test]
    async fn revoke_all_user_access_propagates_token_store_failure() {
        let user_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();

        let mut session_repo = MockUserSessionRepository::new();
        session_repo
            .expect_delete_all_by_user()
            .return_once(|_, _| Box::pin(async { Ok(0) }));

        let mut refresh = MockRefreshTokenRepository::new();
        refresh.expect_revoke_all_for_user().return_once(|_| {
            Box::pin(async {
                Err(ferriskey_security::SecurityError::GenerationError(
                    "boom".to_string(),
                ))
            })
        });

        let mut access = MockAccessTokenRepository::new();
        access
            .expect_revoke_all_for_user()
            .returning(|_| Box::pin(async { Ok(0) }));

        let adapter = TokenRevocationAdapter::new(
            Arc::new(access),
            Arc::new(refresh),
            Arc::new(session_repo),
        );

        assert!(
            adapter
                .revoke_all_user_access(user_id, realm_id)
                .await
                .is_err()
        );
    }
}
