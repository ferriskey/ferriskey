use crate::{
    ApplicationService,
    domain::{
        abyss::federation::{
            entities::{FederationProvider, SyncMode},
            ports::FederationService,
            value_objects::{
                CreateProviderRequest, SyncResult, TestConnectionResult, UpdateProviderRequest,
            },
        },
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
    },
};
use uuid::Uuid;

impl FederationService for ApplicationService {
    async fn create_federation_provider(
        &self,
        request: CreateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        self.federation_service
            .create_federation_provider(request)
            .await
    }

    async fn get_federation_provider(
        &self,
        identity: Identity,
        id: Uuid,
        realm_name: String,
    ) -> Result<FederationProvider, CoreError> {
        self.federation_service
            .get_federation_provider(identity, id, realm_name)
            .await
    }

    async fn update_federation_provider(
        &self,
        id: Uuid,
        request: UpdateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        self.federation_service
            .update_federation_provider(id, request)
            .await
    }

    async fn delete_federation_provider(
        &self,
        identity: Identity,
        id: Uuid,
        realm_name: String,
    ) -> Result<(), CoreError> {
        self.federation_service
            .delete_federation_provider(identity, id, realm_name)
            .await
    }

    async fn list_federation_providers(
        &self,
        realm_id: Uuid,
    ) -> Result<Vec<FederationProvider>, CoreError> {
        self.federation_service
            .list_federation_providers(realm_id)
            .await
    }

    async fn test_federation_connection(
        &self,
        id: Uuid,
    ) -> Result<TestConnectionResult, CoreError> {
        self.federation_service.test_federation_connection(id).await
    }

    async fn sync_federation_users(
        &self,
        id: Uuid,
        mode: SyncMode,
    ) -> Result<SyncResult, CoreError> {
        self.federation_service
            .sync_federation_users(id, mode)
            .await
    }
}
