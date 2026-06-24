use std::sync::Arc;

use chrono::Duration;
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::{entities::app_errors::CoreError, policies::Policy},
    realm::ports::RealmRepository,
    session::{
        entities::{SessionError, UserSession},
        ports::{UserSessionManagementService, UserSessionRepository, UserSessionService},
    },
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
pub struct UserSessionManagementServiceImpl<R, U, P>
where
    R: RealmRepository,
    U: UserSessionRepository,
    P: Policy,
{
    realm_repository: Arc<R>,
    session_repository: Arc<U>,
    policy: Arc<P>,
}

impl<R, U, P> UserSessionManagementServiceImpl<R, U, P>
where
    R: RealmRepository,
    U: UserSessionRepository,
    P: Policy,
{
    pub fn new(realm_repository: Arc<R>, session_repository: Arc<U>, policy: Arc<P>) -> Self {
        Self {
            realm_repository,
            session_repository,
            policy,
        }
    }
}

impl<R, U, P> UserSessionManagementService for UserSessionManagementServiceImpl<R, U, P>
where
    R: RealmRepository,
    U: UserSessionRepository,
    P: Policy,
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

            let has_permission =
                crate::domain::role::entities::permission::Permissions::has_one_of_permissions(
                    &permissions,
                    &[
                        crate::domain::role::entities::permission::Permissions::ManageUsers,
                        crate::domain::role::entities::permission::Permissions::ManageRealm,
                        crate::domain::role::entities::permission::Permissions::ViewUsers,
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
    ) -> Result<(), CoreError> {
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

            let has_permission =
                crate::domain::role::entities::permission::Permissions::has_one_of_permissions(
                    &permissions,
                    &[
                        crate::domain::role::entities::permission::Permissions::ManageUsers,
                        crate::domain::role::entities::permission::Permissions::ManageRealm,
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

        self.session_repository
            .delete(&session_id)
            .await
            .map_err(|_| CoreError::SessionDeleteError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::session::ports::MockUserSessionRepository;
    use uuid::Uuid;

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
            crate::domain::session::entities::SessionState::Active
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
            crate::domain::session::entities::SessionState::Expired
        );
    }
}
