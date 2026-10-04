use ferriskey_compass::{
    entities::CompassFlow,
    ports::CompassService,
    value_objects::{
        DailyActivityStats, DailyActivityStatsFilter, FlowFilter, FlowSortField, FlowStats,
    },
};
use ferriskey_domain::common::pagination::{Page, PageRequest};
use uuid::Uuid;

use crate::{
    application::services::ApplicationService,
    domain::{authentication::value_objects::Identity, common::entities::app_errors::CoreError},
};

impl CompassService for ApplicationService {
    async fn list_flows(
        &self,
        identity: Identity,
        realm_name: String,
        request: PageRequest<FlowFilter, FlowSortField>,
    ) -> Result<Page<CompassFlow>, CoreError> {
        self.compass_service
            .list_flows(identity, realm_name, request)
            .await
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

    async fn get_daily_activity_stats(
        &self,
        identity: Identity,
        realm_name: String,
        filter: DailyActivityStatsFilter,
    ) -> Result<Vec<DailyActivityStats>, CoreError> {
        self.compass_service
            .get_daily_activity_stats(identity, realm_name, filter)
            .await
    }
}
