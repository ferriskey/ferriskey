//! Ports an authorization engine plugs into.
//!
//! These traits are deliberately object-safe, unlike the per-domain
//! `XxxPolicy` traits which use RPITIT. `core::application::services` already
//! stacks up to 26 generic parameters per service; threading an engine
//! parameter through it would multiply every type alias. Boxed futures behind
//! `Arc<dyn _>` keep the wiring flat — the same trade-off `ferriskey-migrate`
//! already makes with `MigrationFuture`.

use std::fmt::Debug;
use std::pin::Pin;

use uuid::Uuid;

use crate::entities::{AccessRequest, Decision, EntityRef, PolicyBundle, PrincipalSlice};
use crate::error::AuthzError;

pub type AuthzFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, AuthzError>> + Send + 'a>>;

/// Answers authorization questions.
///
/// An engine returns `Err` only when it could not compute a decision. A
/// well-formed question whose answer is "no" is `Ok(Decision { allowed: false })`.
pub trait AuthorizationEngine: Send + Sync + Debug {
    fn evaluate<'a>(&'a self, request: &'a AccessRequest) -> AuthzFuture<'a, Decision>;

    /// Evaluate several questions at once.
    ///
    /// The default implementation is sequential; engines that can batch should
    /// override it. Backs the AuthZen `evaluations` endpoint.
    fn evaluate_batch<'a>(
        &'a self,
        requests: &'a [AccessRequest],
    ) -> AuthzFuture<'a, Vec<Decision>> {
        Box::pin(async move {
            let mut decisions = Vec::with_capacity(requests.len());
            for request in requests {
                decisions.push(self.evaluate(request).await?);
            }
            Ok(decisions)
        })
    }
}

/// Supplies the policy set of a realm.
pub trait PolicyStore: Send + Sync + Debug {
    /// Current version, bumped whenever anything the policy set derives from
    /// changes. Cheap enough to poll before deciding to recompile.
    fn version(&self, realm_id: Uuid) -> AuthzFuture<'_, i64>;

    fn load(&self, realm_id: Uuid) -> AuthzFuture<'_, PolicyBundle>;
}

/// Resolves everything an engine needs to know about a principal.
pub trait PrincipalSliceRepository: Send + Sync + Debug {
    fn load<'a>(&'a self, principal: &'a EntityRef) -> AuthzFuture<'a, PrincipalSlice>;
}

/// Resolves containment edges outside the built-in realm/organization/group
/// hierarchy.
///
/// Unused while the hierarchy is the only relation model. It exists so a
/// generic relation-tuple store can be swapped in later without touching
/// policies or call sites.
pub trait RelationResolver: Send + Sync + Debug {
    fn parents<'a>(&'a self, entity: &'a EntityRef) -> AuthzFuture<'a, Vec<EntityRef>>;
}
