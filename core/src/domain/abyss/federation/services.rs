use std::sync::Arc;

use tracing::instrument;
use uuid::Uuid;

use crate::domain::abyss::federation::entities::{FederationProvider, FederationType, SyncMode};
use crate::domain::abyss::federation::ports::{FederationRepository, FederationService};
use crate::domain::abyss::federation::value_objects::{
    CreateProviderRequest, SyncResult, TestConnectionResult, UpdateProviderRequest,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::ports::RealmRepository;
use crate::infrastructure::abyss::federation::ldap::LdapClientImpl;

#[derive(Clone, Debug)]
pub struct FederationServiceImpl<R, F>
where
    R: RealmRepository,
    F: FederationRepository,
{
    federation_repository: Arc<F>,
    realm_repository: Arc<R>,
    ldap_client: LdapClientImpl,
}

impl<R, F> FederationServiceImpl<R, F>
where
    R: RealmRepository,
    F: FederationRepository,
{
    pub fn new(realm_repository: Arc<R>, federation_repository: Arc<F>) -> Self {
        Self {
            realm_repository,
            federation_repository,
            ldap_client: LdapClientImpl,
        }
    }
}

impl<R, F> FederationService for FederationServiceImpl<R, F>
where
    R: RealmRepository,
    F: FederationRepository,
{
    #[instrument(skip(self, request))]
    async fn create_federation_provider(
        &self,
        request: CreateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        // TODO: Validate config based on provider type
        self.federation_repository.create(request).await
    }

    #[instrument(skip(self))]
    async fn get_federation_provider(
        &self,
        identity: Identity,
        id: Uuid,
        realm_name: String,
    ) -> Result<FederationProvider, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let federation = self
            .federation_repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if federation.realm_id != Into::<Uuid>::into(realm.id) {
            return Err(CoreError::Forbidden("forbidden".to_string()));
        }

        Ok(federation)
    }

    #[instrument(skip(self, request))]
    async fn update_federation_provider(
        &self,
        id: Uuid,
        request: UpdateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        self.federation_repository.update(id, request).await
    }

    #[instrument(skip(self))]
    async fn delete_federation_provider(
        &self,
        identity: Identity,
        id: Uuid,
        realm_name: String,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let existing = self
            .federation_repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if existing.realm_id != Into::<Uuid>::into(realm.id) {
            return Err(CoreError::Forbidden("forbidden".to_string()));
        }

        self.federation_repository.delete(id).await
    }

    #[instrument(skip(self))]
    async fn list_federation_providers(
        &self,
        realm_id: Uuid,
    ) -> Result<Vec<FederationProvider>, CoreError> {
        self.federation_repository.list_by_realm(realm_id).await
    }

    #[instrument(skip(self))]
    async fn test_federation_connection(
        &self,
        id: Uuid,
    ) -> Result<TestConnectionResult, CoreError> {
        let provider = self
            .federation_repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

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
        let provider = self
            .federation_repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

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
