use std::sync::Arc;

use chrono::Duration;
use uuid::Uuid;

use crate::auth::Identity;
use crate::common::app_errors::CoreError;
use crate::common::policies::Policy;
use crate::realm::ports::RealmRepository;
use crate::session::entities::{SessionError, UserSession};
use crate::session::ports::{
    TokenRevocationPort, UserSessionManagementService, UserSessionRepository, UserSessionService,
};

#[derive(Clone)]
pub struct UserSessionServiceImpl<U>
where
    U: UserSessionRepository,
{
    pub user_session_repository: U,
}

impl<U> UserSessionServiceImpl<U>
where
    U: UserSessionRepository,
{
    pub fn new(user_session_repository: U) -> Self {
        Self {
            user_session_repository,
        }
    }
}

impl<U> UserSessionService for UserSessionServiceImpl<U>
where
    U: UserSessionRepository,
{
    async fn create_session(
        &self,
        user_id: uuid::Uuid,
        realm_id: uuid::Uuid,
        user_agent: Option<String>,
        ip_address: Option<String>,
        session_duration: Duration,
        soft_expiry_duration: Option<Duration>,
    ) -> Result<UserSession, SessionError> {
        let session = UserSession::new(
            user_id,
            realm_id,
            user_agent,
            ip_address,
            session_duration,
            soft_expiry_duration,
        );

        self.user_session_repository.create(&session).await?;

        Ok(session)
    }
}

#[derive(Clone, Debug)]
pub struct UserSessionManagementServiceImpl<R, U, P, T>
where
    R: RealmRepository,
    U: UserSessionRepository,
    P: Policy,
    T: TokenRevocationPort,
{
    realm_repository: Arc<R>,
    session_repository: Arc<U>,
    policy: Arc<P>,
    token_revocation: Arc<T>,
}

impl<R, U, P, T> UserSessionManagementServiceImpl<R, U, P, T>
where
    R: RealmRepository,
    U: UserSessionRepository,
    P: Policy,
    T: TokenRevocationPort,
{
    pub fn new(
        realm_repository: Arc<R>,
        session_repository: Arc<U>,
        policy: Arc<P>,
        token_revocation: Arc<T>,
    ) -> Self {
        Self {
            realm_repository,
            session_repository,
            policy,
            token_revocation,
        }
    }
}

impl<R, U, P, T> UserSessionManagementService for UserSessionManagementServiceImpl<R, U, P, T>
where
    R: RealmRepository,
    U: UserSessionRepository,
    P: Policy,
    T: TokenRevocationPort,
{
    async fn list_sessions(
        &self,
        identity: Identity,
        realm_name: String,
        user_id: Uuid,
    ) -> Result<Vec<UserSession>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let actor = self.policy.get_user_from_identity(&identity).await?;

        if actor.id != user_id {
            let permissions = self
                .policy
                .get_permission_for_target_realm(&actor, &realm)
                .await?;

            let has_permission = crate::role::permission::Permissions::has_one_of_permissions(
                &permissions,
                &[
                    crate::role::permission::Permissions::ManageUsers,
                    crate::role::permission::Permissions::ManageRealm,
                    crate::role::permission::Permissions::ViewUsers,
                ],
            );

            if !has_permission {
                return Err(CoreError::Forbidden(
                    "insufficient permissions to list sessions".to_string(),
                ));
            }
        }

        let sessions = self
            .session_repository
            .find_all_by_user_and_realm(user_id, realm.id.into())
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(sessions)
    }

    async fn revoke_session(
        &self,
        identity: Identity,
        realm_name: String,
        user_id: Uuid,
        session_id: Uuid,
    ) -> Result<UserSession, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let actor = self.policy.get_user_from_identity(&identity).await?;

        if actor.id != user_id {
            let permissions = self
                .policy
                .get_permission_for_target_realm(&actor, &realm)
                .await?;

            let has_permission = crate::role::permission::Permissions::has_one_of_permissions(
                &permissions,
                &[
                    crate::role::permission::Permissions::ManageUsers,
                    crate::role::permission::Permissions::ManageRealm,
                ],
            );

            if !has_permission {
                return Err(CoreError::Forbidden(
                    "insufficient permissions to revoke sessions".to_string(),
                ));
            }
        }

        let session = self
            .session_repository
            .find_by_id(session_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::SessionNotFound)?;

        let realm_id_uuid: uuid::Uuid = realm.id.into();
        if session.user_id != user_id || session.realm_id != realm_id_uuid {
            return Err(CoreError::SessionNotFound);
        }

        self.token_revocation
            .revoke_session_tokens(session_id)
            .await?;

        self.session_repository
            .delete(&session_id)
            .await
            .map_err(|_| CoreError::SessionDeleteError)?;

        Ok(session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::Identity;
    use crate::client::ports::MockClientRepository;
    use crate::common::policies::FerriskeyPolicy;
    use crate::realm::ports::MockRealmRepository;
    use crate::realm::{Realm, RealmId};
    use crate::session::ports::{MockTokenRevocationPort, MockUserSessionRepository};
    use crate::user::entities::User;
    use crate::user::ports::{MockUserRepository, MockUserRoleRepository};
    use uuid::Uuid;

    type TestPolicy =
        FerriskeyPolicy<MockUserRepository, MockClientRepository, MockUserRoleRepository>;

    type TestManagementService = UserSessionManagementServiceImpl<
        MockRealmRepository,
        MockUserSessionRepository,
        TestPolicy,
        MockTokenRevocationPort,
    >;

    fn make_realm(name: &str) -> Realm {
        Realm {
            id: RealmId::new(Uuid::new_v4()),
            name: name.to_string(),
            display_name: None,
            settings: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn make_user(realm: &Realm) -> User {
        User {
            id: Uuid::new_v4(),
            realm_id: realm.id,
            client_id: None,
            username: "alice".to_string(),
            firstname: None,
            lastname: None,
            email: Some("alice@example.com".to_string()),
            email_verified: true,
            enabled: true,
            roles: None,
            realm: Some(realm.clone()),
            required_actions: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            failed_login_attempts: 0,
            locked_until: None,
        }
    }

    /// Builds the management service with the actor acting on their *own*
    /// sessions, which short-circuits the permission branch — the policy never
    /// touches a repository, so the mocks below stay empty.
    fn build_service(
        realm_repo: MockRealmRepository,
        session_repo: MockUserSessionRepository,
        revoker: MockTokenRevocationPort,
    ) -> TestManagementService {
        let policy = FerriskeyPolicy::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(MockClientRepository::new()),
            Arc::new(MockUserRoleRepository::new()),
        );

        UserSessionManagementServiceImpl::new(
            Arc::new(realm_repo),
            Arc::new(session_repo),
            Arc::new(policy),
            Arc::new(revoker),
        )
    }

    /// FK-007: deleting the `user_sessions` row is not remediation on its own —
    /// the access and refresh tokens minted against it stay valid until their
    /// natural expiry. Revoking a session must reach them.
    #[tokio::test]
    async fn revoke_session_revokes_the_tokens_minted_against_it() {
        let realm = make_realm("test-realm");
        let realm_uuid: Uuid = realm.id.into();
        let user = make_user(&realm);
        let user_id = user.id;
        let session = make_session(user_id, realm_uuid);
        let session_id = session.id;

        let mut realm_repo = MockRealmRepository::new();
        let realm_clone = realm.clone();
        realm_repo
            .expect_get_by_name()
            .return_once(move |_| Box::pin(async move { Ok(Some(realm_clone)) }));

        let mut session_repo = MockUserSessionRepository::new();
        session_repo
            .expect_find_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(session)) }));
        session_repo
            .expect_delete()
            .times(1)
            .return_once(|_| Box::pin(async { Ok(()) }));

        let mut revoker = MockTokenRevocationPort::new();
        revoker
            .expect_revoke_session_tokens()
            .with(mockall::predicate::eq(session_id))
            .times(1)
            .return_once(|_| Box::pin(async { Ok(()) }));

        let svc = build_service(realm_repo, session_repo, revoker);

        let result = svc
            .revoke_session(
                Identity::User(user),
                "test-realm".to_string(),
                user_id,
                session_id,
            )
            .await;

        assert!(result.is_ok(), "revoke_session should succeed");
    }

    /// A cascade that silently fails is the whole bug: the operator is told the
    /// session is gone while the tokens keep working. The row must survive so a
    /// retry is meaningful, and the error must surface.
    #[tokio::test]
    async fn revoke_session_propagates_revocation_failure_and_keeps_the_row() {
        let realm = make_realm("test-realm");
        let realm_uuid: Uuid = realm.id.into();
        let user = make_user(&realm);
        let user_id = user.id;
        let session = make_session(user_id, realm_uuid);
        let session_id = session.id;

        let mut realm_repo = MockRealmRepository::new();
        let realm_clone = realm.clone();
        realm_repo
            .expect_get_by_name()
            .return_once(move |_| Box::pin(async move { Ok(Some(realm_clone)) }));

        let mut session_repo = MockUserSessionRepository::new();
        session_repo
            .expect_find_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(session)) }));
        // The row must NOT be deleted when the cascade failed.
        session_repo.expect_delete().never();

        let mut revoker = MockTokenRevocationPort::new();
        revoker
            .expect_revoke_session_tokens()
            .times(1)
            .return_once(|_| Box::pin(async { Err(CoreError::InternalServerError) }));

        let svc = build_service(realm_repo, session_repo, revoker);

        let result = svc
            .revoke_session(
                Identity::User(user),
                "test-realm".to_string(),
                user_id,
                session_id,
            )
            .await;

        assert!(
            result.is_err(),
            "a failed token cascade must not report success"
        );
    }

    fn make_session(user_id: Uuid, realm_id: Uuid) -> UserSession {
        UserSession {
            id: Uuid::new_v4(),
            user_id,
            realm_id,
            user_agent: Some("Mozilla/5.0".to_string()),
            ip_address: Some("127.0.0.1".to_string()),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + Duration::hours(1),
            last_seen_at: None,
            soft_expiry_duration: None,
        }
    }

    #[tokio::test]
    async fn create_session_calls_repository() {
        let user_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();
        let mut mock = MockUserSessionRepository::new();
        mock.expect_create()
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        let svc = UserSessionServiceImpl::new(mock);
        let result = svc
            .create_session(
                user_id,
                realm_id,
                Some("agent".to_string()),
                Some("1.2.3.4".to_string()),
                Duration::hours(8),
                None,
            )
            .await;

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.user_id, user_id);
        assert_eq!(session.realm_id, realm_id);
        assert!(session.last_seen_at.is_none());
    }

    #[tokio::test]
    async fn session_state_is_active_when_not_expired() {
        let user_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();
        let session = make_session(user_id, realm_id);
        assert_eq!(
            session.get_state(),
            crate::session::entities::SessionState::Active
        );
        assert!(!session.is_expired());
    }

    #[tokio::test]
    async fn session_state_is_expired_when_past_expiry() {
        let user_id = Uuid::new_v4();
        let realm_id = Uuid::new_v4();
        let session = UserSession {
            id: Uuid::new_v4(),
            user_id,
            realm_id,
            user_agent: None,
            ip_address: None,
            created_at: chrono::Utc::now() - Duration::hours(2),
            expires_at: chrono::Utc::now() - Duration::hours(1),
            last_seen_at: None,
            soft_expiry_duration: None,
        };
        assert!(session.is_expired());
        assert_eq!(
            session.get_state(),
            crate::session::entities::SessionState::Expired
        );
    }
}
