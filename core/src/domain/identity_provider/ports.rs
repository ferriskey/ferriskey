use std::future::Future;

use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;

use super::entities::IdentityProvider;
use super::value_objects::{CreateIdentityProviderRequest, UpdateIdentityProviderRequest};

/// Repository trait for Identity Provider persistence
#[cfg_attr(test, mockall::automock)]
pub trait IdentityProviderRepository: Send + Sync {
    /// Creates a new identity provider
    fn create(
        &self,
        request: CreateIdentityProviderRequest,
    ) -> impl Future<Output = Result<IdentityProvider, CoreError>> + Send;

    /// Finds an identity provider by ID
    fn find_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<IdentityProvider>, CoreError>> + Send;

    /// Finds an identity provider by realm and alias
    fn find_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> impl Future<Output = Result<Option<IdentityProvider>, CoreError>> + Send;

    /// Lists all identity providers for a realm
    fn find_by_realm(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<IdentityProvider>, CoreError>> + Send;

    /// Updates an existing identity provider
    fn update(
        &self,
        id: Uuid,
        request: UpdateIdentityProviderRequest,
    ) -> impl Future<Output = Result<IdentityProvider, CoreError>> + Send;

    /// Deletes an identity provider by ID
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Checks if an alias exists in a realm (for uniqueness validation)
    fn exists_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
