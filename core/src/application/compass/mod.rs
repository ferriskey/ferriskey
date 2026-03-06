use ferriskey_compass::{
    entities::CompassFlow,
    ports::CompassService,
    value_objects::{FetchFlowsInput, FlowStats},
};
use uuid::Uuid;

use crate::{
    application::services::ApplicationService,
    domain::{authentication::value_objects::Identity, common::entities::app_errors::CoreError},
};

impl CompassService for ApplicationService {
    async fn fetch_flows(
        &self,
        identity: Identity,
        input: FetchFlowsInput,
    ) -> Result<Vec<CompassFlow>, CoreError> {
        self.compass_service.fetch_flows(identity, input).await
    }

    async fn get_flow(
        &self,
        identity: Identity,
        realm_name: String,
        flow_id: Uuid,
    ) -> Result<CompassFlow, CoreError> {
        self.compass_service
            .get_flow(identity, realm_name, flow_id)
            .await
    }

    async fn get_stats(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<FlowStats, CoreError> {
        self.compass_service.get_stats(identity, realm_name).await
    }
}
