use std::sync::Arc;

use ferriskey_compass::{
    entities::CompassFlow,
    ports::{CompassFlowRepository, CompassFlowStepRepository, CompassPolicy, CompassService},
    value_objects::{FetchFlowsInput, FlowStats},
};
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    realm::ports::RealmRepository,
    user::ports::{UserRepository, UserRoleRepository},
};

#[derive(Clone, Debug)]
pub struct CompassServiceImpl<R, U, C, UR, FR, FS>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    FR: CompassFlowRepository,
    FS: CompassFlowStepRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) flow_repository: Arc<FR>,
    pub(crate) step_repository: Arc<FS>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, FR, FS> CompassServiceImpl<R, U, C, UR, FR, FS>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    FR: CompassFlowRepository,
    FS: CompassFlowStepRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        flow_repository: Arc<FR>,
        step_repository: Arc<FS>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            flow_repository,
            step_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, FR, FS> CompassService for CompassServiceImpl<R, U, C, UR, FR, FS>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    FR: CompassFlowRepository,
    FS: CompassFlowStepRepository,
{
    async fn fetch_flows(
        &self,
        identity: Identity,
        input: FetchFlowsInput,
    ) -> Result<Vec<CompassFlow>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_flows(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let flows = self
            .flow_repository
            .get_flows(realm_id, input.filter)
            .await?;

        Ok(flows)
    }

    async fn get_flow(
        &self,
        identity: Identity,
        realm_name: String,
        flow_id: Uuid,
    ) -> Result<CompassFlow, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_flows(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let mut flow = self
            .flow_repository
            .get_flow_by_id(flow_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if flow.realm_id != realm.id {
            return Err(CoreError::NotFound);
        }

        let steps = self.step_repository.get_steps_for_flow(flow_id).await?;
        flow.steps = steps;

        Ok(flow)
    }

    async fn get_stats(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<FlowStats, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_flows(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let stats = self.flow_repository.get_stats(realm_id).await?;

        Ok(stats)
    }
}
