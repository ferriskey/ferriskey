use crate::domain::{
    abyss::identity_provider::{
        entities::{
            CreateIdentityProviderInput, DeleteIdentityProviderInput, GetIdentityProviderInput,
            IdentityProvider, ListIdentityProvidersInput, UpdateIdentityProviderInput,
        },
        ports::IdentityProviderService,
    },
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
};

use super::ApplicationService;

impl IdentityProviderService for ApplicationService {
    async fn create_identity_provider(
        &self,
        identity: Identity,
        input: CreateIdentityProviderInput,
    ) -> Result<IdentityProvider, CoreError> {
        self.identity_provider_service
            .create_identity_provider(identity, input)
            .await
    }

    async fn get_identity_provider(
        &self,
        identity: Identity,
        input: GetIdentityProviderInput,
    ) -> Result<IdentityProvider, CoreError> {
        self.identity_provider_service
            .get_identity_provider(identity, input)
            .await
    }

    async fn list_identity_providers(
        &self,
        identity: Identity,
        input: ListIdentityProvidersInput,
    ) -> Result<Vec<IdentityProvider>, CoreError> {
        self.identity_provider_service
            .list_identity_providers(identity, input)
            .await
    }

    async fn update_identity_provider(
        &self,
        identity: Identity,
        input: UpdateIdentityProviderInput,
    ) -> Result<IdentityProvider, CoreError> {
        self.identity_provider_service
            .update_identity_provider(identity, input)
            .await
    }

    async fn delete_identity_provider(
        &self,
        identity: Identity,
        input: DeleteIdentityProviderInput,
    ) -> Result<(), CoreError> {
        self.identity_provider_service
            .delete_identity_provider(identity, input)
            .await
    }
}
