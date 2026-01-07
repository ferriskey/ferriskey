use std::sync::Arc;

use tracing::instrument;
use uuid::Uuid;

use crate::domain::abyss::federation::entities::{FederationProvider, FederationType, SyncMode};
use crate::domain::abyss::federation::ports::{
    FederationPolicy, FederationRepository, FederationService,
};
use crate::domain::abyss::federation::value_objects::{
    CreateProviderRequest, SyncResult, TestConnectionResult, UpdateProviderRequest,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::policies::ensure_policy;
use crate::domain::realm::ports::RealmRepository;
use crate::infrastructure::abyss::federation::ldap::LdapClientImpl;

#[derive(Clone, Debug)]
pub struct FederationServiceImpl<R, F, P>
where
    R: RealmRepository,
    F: FederationRepository,
    P: FederationPolicy,
{
    federation_repository: Arc<F>,
    realm_repository: Arc<R>,
    policy: Arc<P>,
    ldap_client: LdapClientImpl,
}

impl<R, F, P> FederationServiceImpl<R, F, P>
where
    R: RealmRepository,
    F: FederationRepository,
    P: FederationPolicy,
{
    pub fn new(realm_repository: Arc<R>, federation_repository: Arc<F>, policy: Arc<P>) -> Self {
        Self {
            realm_repository,
            federation_repository,
            policy,
            ldap_client: LdapClientImpl,
        }
    }
}

impl<R, F, P> FederationService for FederationServiceImpl<R, F, P>
where
    R: RealmRepository,
    F: FederationRepository,
    P: FederationPolicy,
{
    #[instrument(skip(self, identity, request))]
    async fn create_federation_provider(
        &self,
        identity: Identity,
        realm_name: String,
        mut request: CreateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;
        let realm_id = realm.id;

        ensure_policy(
            self.policy
                .can_create_federation_provider(identity, realm)
                .await,
            "insufficient permissions to create provider",
        )?;

        request.realm_id = realm_id.into();

        // TODO: Validate config based on provider type
        self.federation_repository.create(request).await
    }

    #[instrument(skip(self, identity))]
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

        let provider = self
            .federation_repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if provider.realm_id != Into::<Uuid>::into(realm.id) {
            return Err(CoreError::NotFound);
        }

        ensure_policy(
            self.policy
                .can_view_federation_provider(&identity, realm)
                .await,
            "insufficient permissions to view provider",
        )?;

        Ok(provider)
    }

    #[instrument(skip(self, identity, request))]
    async fn update_federation_provider(
        &self,
        identity: Identity,
        realm_name: String,
        id: Uuid,
        request: UpdateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let provider = self
            .federation_repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if provider.realm_id != Into::<Uuid>::into(realm.id) {
            return Err(CoreError::NotFound);
        }

        ensure_policy(
            self.policy
                .can_update_federation_provider(&identity, realm)
                .await,
            "insufficient permissions to update provider",
        )?;

        self.federation_repository.update(id, request).await
    }

    #[instrument(skip(self, identity))]
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

        let provider = self
            .federation_repository
            .get_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if provider.realm_id != Into::<Uuid>::into(realm.id) {
            return Err(CoreError::NotFound);
        }

        ensure_policy(
            self.policy
                .can_delete_federation_provider(&identity, realm)
                .await,
            "insufficient permissions to delete provider",
        )?;

        self.federation_repository.delete(id).await
    }

    #[instrument(skip(self))]
    async fn list_federation_providers(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<FederationProvider>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        self.federation_repository
            .list_by_realm(realm.id.into())
            .await
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
