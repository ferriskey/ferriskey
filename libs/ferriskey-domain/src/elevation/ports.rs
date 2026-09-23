use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::app_errors::CoreError;
use crate::elevation::entities::{Elevation, ElevationId, ElevationProofKind};
use crate::realm::RealmId;
use crate::realm::scope::Unscoped;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ElevationRepository: Send + Sync {
    fn start(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
        session_id: Uuid,
        proof: ElevationProofKind,
        expires_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Elevation, CoreError>> + Send;

    fn find_live(
        &self,
        id: ElevationId,
        user_id: Uuid,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<Option<Unscoped<Elevation>>, CoreError>> + Send;

    fn clear_for_user(&self, user_id: Uuid) -> impl Future<Output = Result<u64, CoreError>> + Send;
}
