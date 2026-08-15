use std::sync::Arc;

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
    async fn revoke_session_tokens(&self, session_id: Uuid) -> Result<(), CoreError> {
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

        tracing::debug!(
            "user {user_id} access revoked: {refresh_revoked} refresh token(s), {access_revoked} access token(s)"
        );

        if let Err(e) = self
            .session_repository
            .delete_all_by_user(user_id, realm_id)
            .await
        {
            warn!(
                "tokens for user {user_id} are revoked but their sessions in realm {realm_id} could not be deleted: {e:?}"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use ferriskey_domain::session::entities::UserSession;
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
        }
    }

    #[tokio::test]
    async fn revoke_session_tokens_hits_both_token_stores() {
        let session_id = Uuid::new_v4();

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

        assert!(adapter.revoke_session_tokens(session_id).await.is_ok());
    }

    #[tokio::test]
    async fn revoke_all_user_access_revokes_tokens_and_drops_sessions() {
        let user_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();
        let sessions = vec![
            make_session(user_id, realm_id),
            make_session(user_id, realm_id),
        ];

        let mut access = MockAccessTokenRepository::new();
        access
            .expect_revoke_all_for_user()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .return_once(|_| Box::pin(async { Ok(3) }));

        let mut refresh = MockRefreshTokenRepository::new();
        refresh
            .expect_revoke_all_for_user()
            .with(mockall::predicate::eq(user_id))
            .times(1)
            .return_once(|_| Box::pin(async { Ok(2) }));

        let mut session_repo = MockUserSessionRepository::new();
        session_repo
            .expect_delete_all_by_user()
            .times(1)
            .return_once(move |_, _| Box::pin(async move { Ok(sessions.len() as u64) }));

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
    }

    /// Token revocation is the security control: if it fails, the caller must be
    /// told, never handed a success it can act on.
    #[tokio::test]
    async fn revoke_all_user_access_propagates_token_store_failure() {
        let user_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();

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
            Arc::new(MockUserSessionRepository::new()),
        );

        assert!(
            adapter
                .revoke_all_user_access(user_id, realm_id)
                .await
                .is_err()
        );
    }
}
