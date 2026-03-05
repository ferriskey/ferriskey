use std::future::Future;

use chrono::{DateTime, Utc};
use ferriskey_domain::{common::app_errors::CoreError, realm::RealmId};
use uuid::Uuid;

use crate::{
    entities::{CompassFlow, CompassFlowStep},
    value_objects::{FlowFilter, FlowStats},
};

#[cfg_attr(test, mockall::automock)]
pub trait CompassFlowRepository: Send + Sync {
    fn create_flow(&self, flow: CompassFlow) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn update_flow_status(
        &self,
        flow_id: Uuid,
        status: String,
        completed_at: DateTime<Utc>,
        duration_ms: Option<i64>,
        user_id: Option<Uuid>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_flows(
        &self,
        realm_id: RealmId,
        filter: FlowFilter,
    ) -> impl Future<Output = Result<Vec<CompassFlow>, CoreError>> + Send;

    fn get_flow_by_id(
        &self,
        flow_id: Uuid,
    ) -> impl Future<Output = Result<Option<CompassFlow>, CoreError>> + Send;

    fn count_flows(
        &self,
        realm_id: RealmId,
        filter: FlowFilter,
    ) -> impl Future<Output = Result<i64, CoreError>> + Send;

    fn purge_old_flows(
        &self,
        older_than: DateTime<Utc>,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;

    fn get_stats(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<FlowStats, CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait CompassFlowStepRepository: Send + Sync {
    fn create_step(
        &self,
        step: CompassFlowStep,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_steps_for_flow(
        &self,
        flow_id: Uuid,
    ) -> impl Future<Output = Result<Vec<CompassFlowStep>, CoreError>> + Send;
}
