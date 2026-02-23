use std::sync::Arc;

use tracing::instrument;

use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::policies::ensure_policy;
use crate::domain::realm::ports::RealmRepository;

use crate::domain::abyss::identity_provider::value_objects::{
    CreateIdentityProviderRequest, UpdateIdentityProviderRequest,
};
use crate::domain::abyss::identity_provider::{
    CreateIdentityProviderInput, DeleteIdentityProviderInput, GetIdentityProviderInput,
    IdentityProvider, ListIdentityProvidersInput, UpdateIdentityProviderInput,
};
use crate::domain::abyss::identity_provider::{
    IdentityProviderPolicy, IdentityProviderRepository, IdentityProviderService,
};

/// Implementation of the IdentityProviderService trait
///
/// Provides business logic for managing identity providers,
/// including authorization checks and validation.
#[derive(Clone, Debug)]
pub struct IdentityProviderServiceImpl<R, P, RR>
where
    R: IdentityProviderRepository,
    P: IdentityProviderPolicy,
    RR: RealmRepository,
{
    identity_provider_repository: Arc<R>,
    identity_provider_policy: Arc<P>,
    realm_repository: Arc<RR>,
}

impl<R, P, RR> IdentityProviderServiceImpl<R, P, RR>
where
    R: IdentityProviderRepository,
    P: IdentityProviderPolicy,
    RR: RealmRepository,
{
    /// Creates a new IdentityProviderServiceImpl
    ///
    /// # Arguments
    /// * `identity_provider_repository` - The identity provider repository for data access
    /// * `identity_provider_policy` - The authorization policy for access control
    /// * `realm_repository` - The realm repository to resolve realm names
    pub fn new(
        identity_provider_repository: Arc<R>,
        identity_provider_policy: Arc<P>,
        realm_repository: Arc<RR>,
    ) -> Self {
        Self {
            identity_provider_repository,
            identity_provider_policy,
            realm_repository,
        }
    }
}

impl<R, P, RR> IdentityProviderService for IdentityProviderServiceImpl<R, P, RR>
where
    R: IdentityProviderRepository,
    P: IdentityProviderPolicy,
    RR: RealmRepository,
{
    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            provider.alias = %input.alias,
        )
    )]
    async fn create_identity_provider(
        &self,
        identity: Identity,
        input: CreateIdentityProviderInput,
    ) -> Result<IdentityProvider, CoreError> {
        // Resolve realm by name
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // Check authorization
        ensure_policy(
            self.identity_provider_policy
                .can_create_identity_provider(&identity, realm.id)
                .await,
            "insufficient permissions to create identity provider",
        )?;

        // Check if alias already exists in realm
        let exists = self
            .identity_provider_repository
            .exists_identity_provider_by_realm_and_alias(realm.id, &input.alias)
            .await?;

        if exists {
            return Err(CoreError::ProviderNameAlreadyExists);
        }

        // Create the identity provider
        let request = CreateIdentityProviderRequest {
            realm_id: realm.id,
            alias: input.alias,
            provider_id: input.provider_id,
            enabled: input.enabled,
            display_name: input.display_name,
            first_broker_login_flow_alias: input.first_broker_login_flow_alias,
            post_broker_login_flow_alias: input.post_broker_login_flow_alias,
            store_token: input.store_token,
            add_read_token_role_on_create: input.add_read_token_role_on_create,
            trust_email: input.trust_email,
            link_only: input.link_only,
            config: input.config,
        };

        self.identity_provider_repository
            .create_identity_provider(request)
            .await
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            provider.alias = %input.alias,
        )
    )]
    async fn get_identity_provider(
        &self,
        identity: Identity,
        input: GetIdentityProviderInput,
    ) -> Result<IdentityProvider, CoreError> {
        // Resolve realm by name
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // Get the identity provider
        let provider = self
            .identity_provider_repository
            .get_identity_provider_by_realm_and_alias(realm.id, &input.alias)
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        // Check authorization
        ensure_policy(
            self.identity_provider_policy
                .can_view_identity_provider(&identity, &provider)
                .await,
            "insufficient permissions to view identity provider",
        )?;

        Ok(provider)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
        )
    )]
    async fn list_identity_providers(
        &self,
        identity: Identity,
        input: ListIdentityProvidersInput,
    ) -> Result<Vec<IdentityProvider>, CoreError> {
        // Resolve realm by name
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // Get all identity providers for the realm
        let providers = self
            .identity_provider_repository
            .list_identity_providers_by_realm(realm.id, None)
            .await?;

        // Filter based on view permission
        let mut accessible_providers = Vec::new();
        for provider in providers {
            if self
                .identity_provider_policy
                .can_view_identity_provider(&identity, &provider)
                .await
                .unwrap_or(false)
            {
                accessible_providers.push(provider);
            }
        }

        Ok(accessible_providers)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            provider.alias = %input.alias,
        )
    )]
    async fn update_identity_provider(
        &self,
        identity: Identity,
        input: UpdateIdentityProviderInput,
    ) -> Result<IdentityProvider, CoreError> {
        // Resolve realm by name
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // Get the identity provider
        let provider = self
            .identity_provider_repository
            .get_identity_provider_by_realm_and_alias(realm.id, &input.alias)
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        // Check authorization
        ensure_policy(
            self.identity_provider_policy
                .can_update_identity_provider(&identity, &provider)
                .await,
            "insufficient permissions to update identity provider",
        )?;

        // Update the identity provider
        let request = UpdateIdentityProviderRequest {
            enabled: input.enabled,
            display_name: input.display_name,
            first_broker_login_flow_alias: input.first_broker_login_flow_alias,
            post_broker_login_flow_alias: input.post_broker_login_flow_alias,
            store_token: input.store_token,
            add_read_token_role_on_create: input.add_read_token_role_on_create,
            trust_email: input.trust_email,
            link_only: input.link_only,
            config: input.config,
        };

        self.identity_provider_repository
            .update_identity_provider(provider.id.into(), request)
            .await
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            provider.alias = %input.alias,
        )
    )]
    async fn delete_identity_provider(
        &self,
        identity: Identity,
        input: DeleteIdentityProviderInput,
    ) -> Result<(), CoreError> {
        // Resolve realm by name
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // Get the identity provider
        let provider = self
            .identity_provider_repository
            .get_identity_provider_by_realm_and_alias(realm.id, &input.alias)
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        // Check authorization
        ensure_policy(
            self.identity_provider_policy
                .can_delete_identity_provider(&identity, &provider)
                .await,
            "insufficient permissions to delete identity provider",
        )?;

        self.identity_provider_repository
            .delete_identity_provider(provider.id.into())
            .await
    }
}
