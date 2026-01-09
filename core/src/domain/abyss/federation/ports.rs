use std::future::Future;
use uuid::Uuid;

use super::entities::{FederationMapping, FederationProvider, SyncMode};
use super::value_objects::{
    CreateProviderRequest, SyncResult, TestConnectionResult, UpdateProviderRequest,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::Realm;

pub trait FederationRepository: Send + Sync {
    // Provider CRUD
    fn create(
        &self,
        request: CreateProviderRequest,
    ) -> impl Future<Output = Result<FederationProvider, CoreError>> + Send;
    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<FederationProvider>, CoreError>> + Send;
    fn update(
        &self,
        id: Uuid,
        request: UpdateProviderRequest,
    ) -> impl Future<Output = Result<FederationProvider, CoreError>> + Send;
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn list_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<FederationProvider>, CoreError>> + Send;

    // Mappings
    fn create_mapping(
        &self,
        mapping: FederationMapping,
    ) -> impl Future<Output = Result<FederationMapping, CoreError>> + Send;
    fn get_mapping(
        &self,
        provider_id: Uuid,
        external_id: &str,
    ) -> impl Future<Output = Result<Option<FederationMapping>, CoreError>> + Send;
    fn list_mappings_by_provider(
        &self,
        provider_id: Uuid,
    ) -> impl Future<Output = Result<Vec<FederationMapping>, CoreError>> + Send;
    fn update_mapping(
        &self,
        mapping: FederationMapping,
    ) -> impl Future<Output = Result<FederationMapping, CoreError>> + Send;
    fn delete_mapping(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait FederationPolicy: Send + Sync {
    fn can_create_federation_provider(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_update_federation_provider(
        &self,
        identity: &Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_view_federation_provider(
        &self,
        identity: &Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_delete_federation_provider(
        &self,
        identity: &Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub trait FederationService: Send + Sync {
    fn create_federation_provider(
        &self,
        identity: Identity,
        realm_name: String,
        request: CreateProviderRequest,
    ) -> impl Future<Output = Result<FederationProvider, CoreError>> + Send;
    fn get_federation_provider(
        &self,
        identity: Identity,
        id: Uuid,
        realm_name: String,
    ) -> impl Future<Output = Result<FederationProvider, CoreError>> + Send;
    fn update_federation_provider(
        &self,
        identity: Identity,
        realm_name: String,
        id: Uuid,
        request: UpdateProviderRequest,
    ) -> impl Future<Output = Result<FederationProvider, CoreError>> + Send;
    fn delete_federation_provider(
        &self,
        identity: Identity,
        id: Uuid,
        realm_name: String,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn list_federation_providers(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> impl Future<Output = Result<Vec<FederationProvider>, CoreError>> + Send;

    fn test_federation_connection(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<TestConnectionResult, CoreError>> + Send;
    fn sync_federation_users(
        &self,
        id: Uuid,
        mode: SyncMode,
    ) -> impl Future<Output = Result<SyncResult, CoreError>> + Send;
}
