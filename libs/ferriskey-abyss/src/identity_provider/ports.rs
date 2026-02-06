use std::future::Future;

use uuid::Uuid;

use ferriskey_domain::auth::Identity;
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::RealmId;

use super::entities::{
    CreateIdentityProviderInput, DeleteIdentityProviderInput, GetIdentityProviderInput,
    IdentityProvider, ListIdentityProvidersInput, UpdateIdentityProviderInput,
};
use super::value_objects::{CreateIdentityProviderRequest, UpdateIdentityProviderRequest};

/// Repository trait for Identity Provider persistence
///
/// Provides data access operations for identity providers.
/// All methods are async and return Result types for error handling.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait IdentityProviderRepository: Send + Sync {
    /// Creates a new identity provider in the database
    fn create_identity_provider(
        &self,
        request: CreateIdentityProviderRequest,
    ) -> impl Future<Output = Result<IdentityProvider, CoreError>> + Send;

    /// Retrieves an identity provider by its unique ID
    fn get_identity_provider_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<IdentityProvider>, CoreError>> + Send;

    /// Retrieves an identity provider by realm and alias
    fn get_identity_provider_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> impl Future<Output = Result<Option<IdentityProvider>, CoreError>> + Send;

    /// Lists all identity providers for a given realm
    fn list_identity_providers_by_realm(
        &self,
        realm_id: RealmId,
        only_enabled: Option<bool>,
    ) -> impl Future<Output = Result<Vec<IdentityProvider>, CoreError>> + Send;

    /// Updates an existing identity provider
    fn update_identity_provider(
        &self,
        id: Uuid,
        request: UpdateIdentityProviderRequest,
    ) -> impl Future<Output = Result<IdentityProvider, CoreError>> + Send;

    /// Deletes an identity provider by ID
    fn delete_identity_provider(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Checks if an alias already exists in a realm
    fn exists_identity_provider_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

/// Service trait for Identity Provider business logic
///
/// Provides high-level operations for managing identity providers,
/// including authorization checks and input validation.
pub trait IdentityProviderService: Send + Sync {
    /// Creates a new identity provider
    fn create_identity_provider(
        &self,
        identity: Identity,
        input: CreateIdentityProviderInput,
    ) -> impl Future<Output = Result<IdentityProvider, CoreError>> + Send;

    /// Retrieves an identity provider by realm and alias
    fn get_identity_provider(
        &self,
        identity: Identity,
        input: GetIdentityProviderInput,
    ) -> impl Future<Output = Result<IdentityProvider, CoreError>> + Send;

    /// Lists all identity providers for a realm
    fn list_identity_providers(
        &self,
        identity: Identity,
        input: ListIdentityProvidersInput,
    ) -> impl Future<Output = Result<Vec<IdentityProvider>, CoreError>> + Send;

    /// Updates an existing identity provider
    fn update_identity_provider(
        &self,
        identity: Identity,
        input: UpdateIdentityProviderInput,
    ) -> impl Future<Output = Result<IdentityProvider, CoreError>> + Send;

    /// Deletes an identity provider
    fn delete_identity_provider(
        &self,
        identity: Identity,
        input: DeleteIdentityProviderInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// Policy trait for Identity Provider authorization
///
/// Defines authorization checks for identity provider operations.
pub trait IdentityProviderPolicy: Send + Sync {
    /// Checks if the identity can create an identity provider in the realm
    fn can_create_identity_provider(
        &self,
        identity: &Identity,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    /// Checks if the identity can view the identity provider
    fn can_view_identity_provider(
        &self,
        identity: &Identity,
        provider: &IdentityProvider,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    /// Checks if the identity can update the identity provider
    fn can_update_identity_provider(
        &self,
        identity: &Identity,
        provider: &IdentityProvider,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    /// Checks if the identity can delete the identity provider
    fn can_delete_identity_provider(
        &self,
        identity: &Identity,
        provider: &IdentityProvider,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
