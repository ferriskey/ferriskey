use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::app_errors::CoreError;
use crate::elevation::entities::{Elevation, ElevationId};
use crate::realm::RealmId;
use crate::realm::scope::Unscoped;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ElevationRepository: Send + Sync {
    fn start(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Elevation, CoreError>> + Send;

    fn consume(
        &self,
        id: ElevationId,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<Option<Unscoped<Elevation>>, CoreError>> + Send;

    fn clear_for_user(&self, user_id: Uuid) -> impl Future<Output = Result<u64, CoreError>> + Send;
}
