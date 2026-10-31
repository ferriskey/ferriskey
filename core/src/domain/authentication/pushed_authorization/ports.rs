use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::authentication::pushed_authorization::entities::{
    AuthorizationParameters, PushedAuthorizationRequest, RequestUriDigest,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;

/// Keeps pushed authorization requests until `/auth` consumes them.
pub trait PushedAuthorizationRequestRepository: Send + Sync {
    fn store(
        &self,
        request: &PushedAuthorizationRequest,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Removes the request and returns its parameters, only if it was pushed
    /// by `client_id` of `realm_id` and has not expired at `now`. Anything
    /// else leaves the row untouched and returns `None`.
    fn take(
        &self,
        realm_id: RealmId,
        client_id: Uuid,
        digest: &RequestUriDigest,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<Option<AuthorizationParameters>, CoreError>> + Send;

    fn delete_expired(
        &self,
        before: DateTime<Utc>,
        limit: u64,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;
}
