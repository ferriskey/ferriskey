use std::future::Future;

use crate::domain::abyss::entities::{Provider, ProviderId, ProviderMapping, ProviderMappingId};
use crate::domain::abyss::value_objects::{
    CreateProviderInput, CreateProviderMappingInput, DeleteProviderInput,
    DeleteProviderMappingInput, GetProviderInput, GetProviderMappingsByProviderInput,
    GetProvidersByRealmInput, ToggleProviderInput, UpdateProviderInput,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;

/// Service trait for managing external identity providers
///
/// Provides high-level operations for CRUD and management of identity providers.
/// All operations requiring authorization accept an `Identity` parameter.
pub trait ProviderService: Send + Sync {
    /// Creates a new identity provider
    ///
    /// # Arguments
    /// * `identity` - The identity performing the operation
    /// * `input` - Provider configuration
    ///
    /// # Returns
    /// The created provider on success
    fn create_provider(
        &self,
        identity: Identity,
        input: CreateProviderInput,
    ) -> impl Future<Output = Result<Provider, CoreError>> + Send;

    /// Retrieves a provider by ID
    ///
    /// # Arguments
    /// * `identity` - The identity performing the operation
    /// * `input` - Contains the provider ID
    ///
    /// # Returns
    /// The provider if found and accessible
    fn get_provider(
        &self,
        identity: Identity,
        input: GetProviderInput,
    ) -> impl Future<Output = Result<Provider, CoreError>> + Send;

    /// Lists all providers for a realm
    ///
    /// # Arguments
    /// * `identity` - The identity performing the operation
    /// * `input` - Contains the realm ID
    ///
    /// # Returns
    /// List of providers the identity can access
    fn list_providers_by_realm(
        &self,
        identity: Identity,
        input: GetProvidersByRealmInput,
    ) -> impl Future<Output = Result<Vec<Provider>, CoreError>> + Send;

    /// Lists only enabled providers for a realm (public, no auth required)
    ///
    /// This is used on the login page to show available identity providers.
    ///
    /// # Arguments
    /// * `realm_id` - The realm to get providers for
    ///
    /// # Returns
    /// List of enabled providers
    fn list_enabled_providers(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<Provider>, CoreError>> + Send;

    /// Updates an existing provider
    ///
    /// # Arguments
    /// * `identity` - The identity performing the operation
    /// * `input` - Contains the provider ID and fields to update
    ///
    /// # Returns
    /// The updated provider
    fn update_provider(
        &self,
        identity: Identity,
        input: UpdateProviderInput,
    ) -> impl Future<Output = Result<Provider, CoreError>> + Send;

    /// Deletes a provider
    ///
    /// # Arguments
    /// * `identity` - The identity performing the operation
    /// * `input` - Contains the provider ID to delete
    fn delete_provider(
        &self,
        identity: Identity,
        input: DeleteProviderInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Enables or disables a provider
    ///
    /// # Arguments
    /// * `identity` - The identity performing the operation
    /// * `input` - Contains the provider ID and new enabled status
    ///
    /// # Returns
    /// The updated provider
    fn toggle_provider(
        &self,
        identity: Identity,
        input: ToggleProviderInput,
    ) -> impl Future<Output = Result<Provider, CoreError>> + Send;

    /// Creates a new attribute mapping for a provider
    ///
    /// # Arguments
    /// * `identity` - The identity performing the operation
    /// * `input` - Mapping configuration
    ///
    /// # Returns
    /// The created mapping
    fn create_provider_mapping(
        &self,
        identity: Identity,
        input: CreateProviderMappingInput,
    ) -> impl Future<Output = Result<ProviderMapping, CoreError>> + Send;

    /// Gets all mappings for a provider
    ///
    /// # Arguments
    /// * `identity` - The identity performing the operation
    /// * `input` - Contains the provider ID
    ///
    /// # Returns
    /// List of mappings for the provider
    fn list_provider_mappings(
        &self,
        identity: Identity,
        input: GetProviderMappingsByProviderInput,
    ) -> impl Future<Output = Result<Vec<ProviderMapping>, CoreError>> + Send;

    /// Deletes an attribute mapping
    ///
    /// # Arguments
    /// * `identity` - The identity performing the operation
    /// * `input` - Contains the mapping ID to delete
    fn delete_provider_mapping(
        &self,
        identity: Identity,
        input: DeleteProviderMappingInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// Policy trait for authorization checks on provider operations
///
/// Determines whether an identity has permission to perform specific
/// operations on providers and their mappings.
pub trait ProviderPolicy: Send + Sync {
    /// Checks if the identity can create providers in a realm
    ///
    /// # Arguments
    /// * `identity` - The identity to check
    /// * `realm_id` - The target realm
    fn can_create_provider(
        &self,
        identity: &Identity,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    /// Checks if the identity can view a specific provider
    ///
    /// # Arguments
    /// * `identity` - The identity to check
    /// * `provider` - The provider to check access for
    fn can_view_provider(
        &self,
        identity: &Identity,
        provider: &Provider,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    /// Checks if the identity can update a specific provider
    ///
    /// # Arguments
    /// * `identity` - The identity to check
    /// * `provider` - The provider to check access for
    fn can_update_provider(
        &self,
        identity: &Identity,
        provider: &Provider,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    /// Checks if the identity can delete a specific provider
    ///
    /// # Arguments
    /// * `identity` - The identity to check
    /// * `provider` - The provider to check access for
    fn can_delete_provider(
        &self,
        identity: &Identity,
        provider: &Provider,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

/// Repository trait for provider persistence
///
/// Provides low-level data access operations for providers and mappings.
/// Implementations should handle database interactions.
#[cfg_attr(test, mockall::automock)]
pub trait ProviderRepository: Send + Sync {
    /// Creates a new provider in the database
    ///
    /// # Arguments
    /// * `provider` - The provider to create
    ///
    /// # Returns
    /// The created provider with any database-generated fields
    fn create_provider(
        &self,
        provider: &Provider,
    ) -> impl Future<Output = Result<Provider, CoreError>> + Send;

    /// Retrieves a provider by its ID
    ///
    /// # Arguments
    /// * `id` - The provider ID
    ///
    /// # Returns
    /// The provider if found, None otherwise
    fn get_provider_by_id(
        &self,
        id: ProviderId,
    ) -> impl Future<Output = Result<Option<Provider>, CoreError>> + Send;

    /// Retrieves all providers for a realm
    ///
    /// # Arguments
    /// * `realm_id` - The realm ID
    ///
    /// # Returns
    /// List of all providers in the realm
    fn list_providers_by_realm(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<Provider>, CoreError>> + Send;

    /// Retrieves a provider by realm and name
    ///
    /// # Arguments
    /// * `realm_id` - The realm ID
    /// * `name` - The provider name
    ///
    /// # Returns
    /// The provider if found, None otherwise
    fn get_provider_by_realm_and_name(
        &self,
        realm_id: RealmId,
        name: String,
    ) -> impl Future<Output = Result<Option<Provider>, CoreError>> + Send;

    /// Updates an existing provider
    ///
    /// # Arguments
    /// * `provider` - The provider with updated fields
    ///
    /// # Returns
    /// The updated provider
    fn update_provider(
        &self,
        provider: &Provider,
    ) -> impl Future<Output = Result<Provider, CoreError>> + Send;

    /// Deletes a provider by ID
    ///
    /// # Arguments
    /// * `id` - The provider ID to delete
    fn delete_provider(&self, id: ProviderId)
    -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Lists all enabled providers for a realm
    ///
    /// # Arguments
    /// * `realm_id` - The realm ID
    ///
    /// # Returns
    /// List of enabled providers
    fn list_enabled_providers_by_realm(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<Provider>, CoreError>> + Send;

    /// Creates a new attribute mapping
    ///
    /// # Arguments
    /// * `mapping` - The mapping to create
    ///
    /// # Returns
    /// The created mapping
    fn create_provider_mapping(
        &self,
        mapping: &ProviderMapping,
    ) -> impl Future<Output = Result<ProviderMapping, CoreError>> + Send;

    /// Gets all mappings for a provider
    ///
    /// # Arguments
    /// * `provider_id` - The provider ID
    ///
    /// # Returns
    /// List of mappings for the provider
    fn list_provider_mappings_by_provider(
        &self,
        provider_id: ProviderId,
    ) -> impl Future<Output = Result<Vec<ProviderMapping>, CoreError>> + Send;

    /// Deletes a mapping by ID
    ///
    /// # Arguments
    /// * `id` - The mapping ID to delete
    fn delete_provider_mapping(
        &self,
        id: ProviderMappingId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Gets a mapping by ID
    ///
    /// # Arguments
    /// * `id` - The mapping ID
    ///
    /// # Returns
    /// The mapping if found, None otherwise
    fn get_provider_mapping_by_id(
        &self,
        id: ProviderMappingId,
    ) -> impl Future<Output = Result<Option<ProviderMapping>, CoreError>> + Send;
}
