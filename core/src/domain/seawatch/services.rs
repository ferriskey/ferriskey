use std::sync::Arc;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    realm::ports::RealmRepository,
    seawatch::{
        SecurityEvent, SecurityEventFilter, SecurityEventPolicy, SecurityEventRepository,
        ports::SecurityEventService, value_objects::FetchEventsInput,
    },
    user::ports::{UserRepository, UserRoleRepository},
};

#[derive(Clone)]
pub struct SecurityEventServiceImpl<R, U, C, UR, SE>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    SE: SecurityEventRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) security_event_repository: Arc<SE>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, SE> SecurityEventServiceImpl<R, U, C, UR, SE>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    SE: SecurityEventRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        security_event_repository: Arc<SE>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            security_event_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, SE> SecurityEventService for SecurityEventServiceImpl<R, U, C, UR, SE>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    SE: SecurityEventRepository,
{
    async fn fetch_events(
        &self,
        identity: Identity,
        input: FetchEventsInput,
    ) -> Result<Vec<SecurityEvent>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_events(identity, realm).await,
            "insufficient permissions",
        )?;

        let security_events = self
            .security_event_repository
            .get_events(
                realm_id,
                SecurityEventFilter {
                    ..Default::default()
                },
            )
            .await?;

        Ok(security_events)
    }
}
