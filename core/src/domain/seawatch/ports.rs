use std::future::Future;
use uuid::Uuid;

use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::Realm;

use super::entities::SecurityEvent;
use super::value_objects::SecurityEventFilter;

pub trait SecurityEventRepository: Send + Sync {
    fn store_event(
        &self,
        event: SecurityEvent,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn get_events(
        &self,
        realm_id: Uuid,
        filter: SecurityEventFilter,
    ) -> impl Future<Output = Result<Vec<SecurityEvent>, CoreError>> + Send;
    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<SecurityEvent, CoreError>> + Send;
    fn count_events(
        &self,
        realm_id: Uuid,
        filter: SecurityEventFilter,
    ) -> impl Future<Output = Result<i64, CoreError>> + Send;
}

pub trait SecurityEventPolicy: Send + Sync {
    fn can_view_events(
        &self,
        identity: Identity,
        realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_export_events(
        &self,
        identity: Identity,
        realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
