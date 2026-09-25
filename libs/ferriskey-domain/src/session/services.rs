use std::sync::Arc;

use chrono::Duration;
use uuid::Uuid;

use crate::auth::Identity;
use crate::common::app_errors::CoreError;
use crate::common::policies::Policy;
use crate::realm::ports::RealmRepository;
use crate::realm::scope::{RealmScope, UnscopedOption};
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
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        let actor = self.policy.get_user_from_identity(&identity).await?;

        if actor.id != user_id {
            let permissions = self
                .policy
                .get_permission_for_target_realm(&actor, scope.realm())
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
            .find_all_by_user_and_realm(user_id, scope.id().into())
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
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;

        let actor = self.policy.get_user_from_identity(&identity).await?;

        if actor.id != user_id {
            let permissions = self
                .policy
                .get_permission_for_target_realm(&actor, scope.realm())
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
            .in_realm(&scope)?
            .ok_or(CoreError::SessionNotFound)?;

        if session.get().user_id != user_id {
            return Err(CoreError::SessionNotFound);
        }

        // Cut the SSO cookie first: while it still resolves, `/auth` can mint
        // tokens bound to this session that the revocation below would miss.
        // The row stays until the tokens are gone so a failed cascade can be
        // retried.
        self.session_repository
            .clear_sso_token_hash(&session)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.token_revocation
            .revoke_session_tokens(&session)
            .await?;

        self.session_repository
            .delete(&session)
            .await
            .map_err(|_| CoreError::SessionDeleteError)?;

        Ok(session.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::Identity;
    use crate::realm::ports::MockRealmRepository;
    use crate::realm::scope::Unscoped;
    use crate::realm::{Realm, RealmId};
    use crate::role::permission::Permissions;
    use crate::session::ports::{MockTokenRevocationPort, MockUserSessionRepository};
    use crate::user::entities::User;
    use std::collections::HashSet;
    use uuid::Uuid;

    /// The engine lives in `ferriskey-authz`, which the kernel cannot depend
    /// on; these tests only need the caller resolved from its identity.
    struct TestPolicy;

    impl Policy for TestPolicy {
        async fn get_user_from_identity(&self, identity: &Identity) -> Result<User, CoreError> {
            match identity {
                Identity::User(user) => Ok(user.clone()),
                Identity::Client(_) => Err(CoreError::Forbidden("no service account".to_string())),
            }
        }

        async fn get_user_permissions(
            &self,
            _user: &User,
        ) -> Result<HashSet<Permissions>, CoreError> {
            Ok(HashSet::new())
        }

        async fn get_permission_for_target_realm(
            &self,
            _user: &User,
            _target_realm: &Realm,
        ) -> Result<HashSet<Permissions>, CoreError> {
            Ok(HashSet::new())
        }

        fn can_access_realm(&self, user_realm: &Realm, target_realm: &Realm) -> bool {
            user_realm.id == target_realm.id
        }
    }

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
            locale: None,
        }
    }

    fn build_service(
        realm_repo: MockRealmRepository,
        session_repo: MockUserSessionRepository,
        revoker: MockTokenRevocationPort,
    ) -> TestManagementService {
        UserSessionManagementServiceImpl::new(
            Arc::new(realm_repo),
            Arc::new(session_repo),
            Arc::new(TestPolicy),
            Arc::new(revoker),
        )
    }

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

        let mut seq = mockall::Sequence::new();
        let mut session_repo = MockUserSessionRepository::new();
        let mut revoker = MockTokenRevocationPort::new();
        session_repo
            .expect_find_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(Unscoped::new(session))) }));
        session_repo
            .expect_clear_sso_token_hash()
            .withf(move |session| session.get().id == session_id)
            .times(1)
            .in_sequence(&mut seq)
            .return_once(|_| Box::pin(async { Ok(()) }));
        revoker
            .expect_revoke_session_tokens()
            .withf(move |session| session.get().id == session_id)
            .times(1)
            .in_sequence(&mut seq)
            .return_once(|_| Box::pin(async { Ok(()) }));
        session_repo
            .expect_delete()
            .times(1)
            .in_sequence(&mut seq)
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
            .return_once(move |_| Box::pin(async move { Ok(Some(Unscoped::new(session))) }));
        session_repo
            .expect_clear_sso_token_hash()
            .times(1)
            .return_once(|_| Box::pin(async { Ok(()) }));
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

    #[tokio::test]
    async fn revoke_session_stops_when_the_sso_cookie_cannot_be_cut() {
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
            .return_once(move |_| Box::pin(async move { Ok(Some(Unscoped::new(session))) }));
        session_repo
            .expect_clear_sso_token_hash()
            .times(1)
            .return_once(|_| Box::pin(async { Err(SessionError::UpdateError) }));
        session_repo.expect_delete().never();

        let mut revoker = MockTokenRevocationPort::new();
        revoker.expect_revoke_session_tokens().never();

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
            "a session whose SSO cookie still resolves must not be reported revoked"
        );
    }

    #[tokio::test]
    async fn revoke_session_refuses_a_session_of_another_realm() {
        let realm = make_realm("test-realm");
        let user = make_user(&realm);
        let user_id = user.id;
        let session = make_session(user_id, Uuid::new_v4());
        let session_id = session.id;

        let mut realm_repo = MockRealmRepository::new();
        let realm_clone = realm.clone();
        realm_repo
            .expect_get_by_name()
            .return_once(move |_| Box::pin(async move { Ok(Some(realm_clone)) }));

        let mut session_repo = MockUserSessionRepository::new();
        session_repo
            .expect_find_by_id()
            .return_once(move |_| Box::pin(async move { Ok(Some(Unscoped::new(session))) }));
        session_repo.expect_clear_sso_token_hash().never();
        session_repo.expect_delete().never();

        let mut revoker = MockTokenRevocationPort::new();
        revoker.expect_revoke_session_tokens().never();

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
            matches!(result, Err(CoreError::NotFound)),
            "a session of another realm must be refused as absent"
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
            sso_token_hash: None,
            persistent: false,
            authenticated_at: chrono::Utc::now(),
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

    #[test]
    fn only_a_persistent_session_gives_its_cookie_a_max_age() {
        let mut session = UserSession::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            None,
            Duration::hours(1),
            None,
        );
        assert_eq!(
            session.cookie_max_age(),
            None,
            "a session nobody asked to remember must die with the browser"
        );

        session.persistent = true;
        let max_age = session
            .cookie_max_age()
            .expect("a remembered session outlives the browser");
        assert!((3590..=3600).contains(&max_age), "got {max_age}");
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
            sso_token_hash: None,
            persistent: false,
            authenticated_at: chrono::Utc::now() - Duration::hours(2),
        };
        assert!(session.is_expired());
        assert_eq!(
            session.get_state(),
            crate::session::entities::SessionState::Expired
        );
    }
}
