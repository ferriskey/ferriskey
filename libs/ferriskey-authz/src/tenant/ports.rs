use std::fmt::Debug;

use ferriskey_domain::auth::Identity;
use ferriskey_domain::common::app_errors::{CoreError, PolicyIssue};
use ferriskey_domain::realm::Realm;
use ferriskey_domain::realm::scope::RealmScope;
use uuid::Uuid;

use super::entities::{AuthzSchema, AuthzState, Policy, PolicyTemplate, RelationTuple};
use super::value_objects::{
    CreatePolicyInput, CreateTemplateInput, ReplaceSchemaInput, UpdatePolicyInput,
    UpdateTemplateInput,
};
use crate::entities::EntityRef;

/// Every write method takes the `expected_version` the caller read, bumps the
/// realm's version in the same transaction, and fails with
/// `CoreError::AuthorizationConflict` when someone else wrote in between.
/// Without that check two writes, each valid alone, could store a rule set
/// that was never validated as a whole.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait AuthzStateRepository: Send + Sync {
    /// A realm that never touched authorization reads as `AuthzState::default()`.
    fn get(&self, realm: &RealmScope)
    -> impl Future<Output = Result<AuthzState, CoreError>> + Send;

    fn set_enabled(
        &self,
        realm: &RealmScope,
        enabled: bool,
    ) -> impl Future<Output = Result<AuthzState, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait AuthzSchemaRepository: Send + Sync {
    fn get(
        &self,
        realm: &RealmScope,
    ) -> impl Future<Output = Result<Option<AuthzSchema>, CoreError>> + Send;

    fn replace(
        &self,
        realm: &RealmScope,
        schema: AuthzSchema,
        expected_version: i64,
    ) -> impl Future<Output = Result<AuthzSchema, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait PolicyRepository: Send + Sync {
    fn list(
        &self,
        realm: &RealmScope,
    ) -> impl Future<Output = Result<Vec<Policy>, CoreError>> + Send;

    /// Inserts the policy, or replaces the one with the same id.
    fn save(
        &self,
        realm: &RealmScope,
        policy: Policy,
        expected_version: i64,
    ) -> impl Future<Output = Result<Policy, CoreError>> + Send;

    fn delete(
        &self,
        realm: &RealmScope,
        id: Uuid,
        expected_version: i64,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait PolicyTemplateRepository: Send + Sync {
    fn list(
        &self,
        realm: &RealmScope,
    ) -> impl Future<Output = Result<Vec<PolicyTemplate>, CoreError>> + Send;

    /// Inserts the template, or replaces the one with the same id.
    fn save(
        &self,
        realm: &RealmScope,
        template: PolicyTemplate,
        expected_version: i64,
    ) -> impl Future<Output = Result<PolicyTemplate, CoreError>> + Send;

    fn delete(
        &self,
        realm: &RealmScope,
        id: Uuid,
        expected_version: i64,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// Tuples are read on every decision rather than cached, so writing them does
/// not move the realm's version.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait RelationTupleRepository: Send + Sync {
    /// Applies every write and delete, or none.
    fn write(
        &self,
        realm: &RealmScope,
        writes: Vec<RelationTuple>,
        deletes: Vec<RelationTuple>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Forgets an entity that no longer exists, so a reused id inherits nothing.
    fn delete_by_subject(
        &self,
        realm: &RealmScope,
        subject: &EntityRef,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;
}

/// Checks a rule set the way the engine will read it.
///
/// Object-safe and synchronous, like `AuthorizationEngine`: the Cedar adapter
/// sits behind an `Arc<dyn PolicyCompiler>` so it does not add a generic
/// parameter to every service alias, and validation is pure CPU work.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait PolicyCompiler: Send + Sync + Debug {
    fn validate_schema(&self, schema: &AuthzSchema) -> Result<(), Vec<PolicyIssue>>;

    /// Validates `policies` and `templates` together, against `schema` when
    /// there is one. Callers pass the enabled policies only.
    #[expect(
        clippy::needless_lifetimes,
        reason = "mockall cannot mock an elided lifetime inside an Option"
    )]
    fn validate_policies<'a>(
        &self,
        schema: Option<&'a AuthzSchema>,
        policies: &[Policy],
        templates: &[PolicyTemplate],
    ) -> Result<(), Vec<PolicyIssue>>;
}

/// Who may read and change a realm's authorization rules.
pub trait TenantAuthorizationPolicy: Send + Sync {
    fn can_view_authorization(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_manage_authorization(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub trait AuthzManagementService: Send + Sync {
    fn get_state(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> impl Future<Output = Result<AuthzState, CoreError>> + Send;

    fn set_enabled(
        &self,
        identity: Identity,
        realm_name: String,
        enabled: bool,
    ) -> impl Future<Output = Result<AuthzState, CoreError>> + Send;

    fn get_schema(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> impl Future<Output = Result<Option<AuthzSchema>, CoreError>> + Send;

    fn replace_schema(
        &self,
        identity: Identity,
        input: ReplaceSchemaInput,
    ) -> impl Future<Output = Result<AuthzSchema, CoreError>> + Send;

    fn list_policies(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> impl Future<Output = Result<Vec<Policy>, CoreError>> + Send;

    fn get_policy(
        &self,
        identity: Identity,
        realm_name: String,
        id: Uuid,
    ) -> impl Future<Output = Result<Policy, CoreError>> + Send;

    fn create_policy(
        &self,
        identity: Identity,
        input: CreatePolicyInput,
    ) -> impl Future<Output = Result<Policy, CoreError>> + Send;

    fn update_policy(
        &self,
        identity: Identity,
        input: UpdatePolicyInput,
    ) -> impl Future<Output = Result<Policy, CoreError>> + Send;

    fn delete_policy(
        &self,
        identity: Identity,
        realm_name: String,
        id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn list_templates(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> impl Future<Output = Result<Vec<PolicyTemplate>, CoreError>> + Send;

    fn create_template(
        &self,
        identity: Identity,
        input: CreateTemplateInput,
    ) -> impl Future<Output = Result<PolicyTemplate, CoreError>> + Send;

    fn update_template(
        &self,
        identity: Identity,
        input: UpdateTemplateInput,
    ) -> impl Future<Output = Result<PolicyTemplate, CoreError>> + Send;

    fn delete_template(
        &self,
        identity: Identity,
        realm_name: String,
        id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
