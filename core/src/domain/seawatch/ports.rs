use std::future::Future;
use uuid::Uuid;

use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::{Realm, RealmId};

use super::entities::SecurityEvent;
use super::hashing::VerifyResult;
use super::value_objects::{FetchEventsInput, SecurityEventFilter};

pub trait SecurityEventService: Send + Sync {
    fn fetch_events(
        &self,
        identity: Identity,
        input: FetchEventsInput,
    ) -> impl Future<Output = Result<Vec<SecurityEvent>, CoreError>> + Send;

    fn verify_realm_chain(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> impl Future<Output = Result<VerifyResult, CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait SecurityEventRepository: Send + Sync {
    /// Store an event that has already had its `event_hash` and `prev_hash` populated.
    fn store_event(
        &self,
        event: SecurityEvent,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Atomically retrieve the current chain head hash for `realm_id`, compute and
    /// store the new event with correct `event_hash` / `prev_hash` fields.
    fn store_event_chained(
        &self,
        event: SecurityEvent,
    ) -> impl Future<Output = Result<SecurityEvent, CoreError>> + Send;

    fn get_events(
        &self,
        realm_id: RealmId,
        filter: SecurityEventFilter,
    ) -> impl Future<Output = Result<Vec<SecurityEvent>, CoreError>> + Send;

    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<SecurityEvent, CoreError>> + Send;

    fn count_events(
        &self,
        realm_id: Uuid,
        filter: SecurityEventFilter,
    ) -> impl Future<Output = Result<i64, CoreError>> + Send;

    /// Return all events for the realm ordered by insertion (created_at ASC) for
    /// chain verification.
    fn get_events_ordered_for_verification(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<SecurityEvent>, CoreError>> + Send;
}

pub trait SecurityEventPolicy: Send + Sync {
    fn can_view_events(
        &self,
        identity: &Identity,
        realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_export_events(
        &self,
        identity: &Identity,
        realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
