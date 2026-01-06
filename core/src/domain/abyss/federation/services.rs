use std::sync::Arc;

use tracing::instrument;
use uuid::Uuid;

use crate::domain::abyss::federation::entities::{FederationProvider, FederationType, SyncMode};
use crate::domain::abyss::federation::ports::{FederationRepository, FederationService};
use crate::domain::abyss::federation::value_objects::{
    CreateProviderRequest, SyncResult, TestConnectionResult, UpdateProviderRequest,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::infrastructure::abyss::federation::ldap::LdapClientImpl;

#[derive(Clone, Debug)]
pub struct FederationServiceImpl<R>
where
    R: FederationRepository,
{
    repository: Arc<R>,
    ldap_client: LdapClientImpl,
}

impl<R> FederationServiceImpl<R>
where
    R: FederationRepository,
{
    pub fn new(repository: Arc<R>) -> Self {
        Self {
            repository,
            ldap_client: LdapClientImpl,
        }
    }
}

impl<R> FederationService for FederationServiceImpl<R>
where
    R: FederationRepository,
{
    #[instrument(skip(self, request))]
    async fn create_federation_provider(
        &self,
        request: CreateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        // TODO: Validate config based on provider type
        self.repository.create(request).await
    }

    #[instrument(skip(self))]
    async fn get_federation_provider(&self, id: Uuid) -> Result<FederationProvider, CoreError> {
        self.repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    #[instrument(skip(self, request))]
    async fn update_federation_provider(
        &self,
        id: Uuid,
        request: UpdateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        self.repository.update(id, request).await
    }

    #[instrument(skip(self))]
    async fn delete_federation_provider(&self, id: Uuid) -> Result<(), CoreError> {
        self.repository.delete(id).await
    }

    #[instrument(skip(self))]
    async fn list_federation_providers(
        &self,
        realm_id: Uuid,
    ) -> Result<Vec<FederationProvider>, CoreError> {
        self.repository.list_by_realm(realm_id).await
    }

    #[instrument(skip(self))]
    async fn test_federation_connection(
        &self,
        id: Uuid,
    ) -> Result<TestConnectionResult, CoreError> {
        let provider = self.get_federation_provider(id).await?;

        match provider.provider_type {
            FederationType::Ldap | FederationType::ActiveDirectory => {
                self.ldap_client.test_connection(&provider).await
            }
            _ => Ok(TestConnectionResult {
                success: false,
                message: "Provider type not supported for connection testing".to_string(),
                details: None,
            }),
        }
    }

    #[instrument(skip(self))]
    async fn sync_federation_users(
        &self,
        id: Uuid,
        mode: SyncMode,
    ) -> Result<SyncResult, CoreError> {
        let provider = self.get_federation_provider(id).await?;

        // Basic stub for sync - just searching for now
        match provider.provider_type {
            FederationType::Ldap | FederationType::ActiveDirectory => {
                let users = self.ldap_client.search_users(&provider, None).await?;
                // TODO: Actual sync logic (create/update local users)
                Ok(SyncResult {
                    users_found: users.len() as u32,
                    created: 0,
                    updated: 0,
                    failed: 0,
                    errors: vec![],
                })
            }
            _ => Err(CoreError::Configuration(
                "Provider type does not support sync".to_string(),
            )),
        }
    }
}
