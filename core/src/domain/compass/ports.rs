use std::future::Future;

use uuid::Uuid;

use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::Realm;

use ferriskey_compass::{
    entities::CompassFlow,
    value_objects::{FetchFlowsInput, FlowStats},
};

pub trait CompassService: Send + Sync {
    fn fetch_flows(
        &self,
        identity: Identity,
        input: FetchFlowsInput,
    ) -> impl Future<Output = Result<Vec<CompassFlow>, CoreError>> + Send;

    fn get_flow(
        &self,
        identity: Identity,
        realm_name: String,
        flow_id: Uuid,
    ) -> impl Future<Output = Result<CompassFlow, CoreError>> + Send;

    fn get_stats(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> impl Future<Output = Result<FlowStats, CoreError>> + Send;
}

pub trait CompassPolicy: Send + Sync {
    fn can_view_flows(
        &self,
        identity: &Identity,
        realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
