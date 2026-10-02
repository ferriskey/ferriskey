use std::future::Future;
use uuid::Uuid;

use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::RealmId;

use super::entities::ConsentDecision;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ConsentDecisionRepository: Send + Sync {
    fn find(
        &self,
        realm_id: RealmId,
        user_id: Uuid,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Option<ConsentDecision>, CoreError>> + Send;

    fn upsert(
        &self,
        decision: ConsentDecision,
    ) -> impl Future<Output = Result<ConsentDecision, CoreError>> + Send;

    fn get_client_consent_required(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn get_consent_ttl_days(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Option<i32>, CoreError>> + Send;
}
