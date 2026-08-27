use std::collections::HashSet;

use crate::{
    application::services::ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        client::{
            entities::{
                Client, CreateClientInput, CreatePostLogoutRedirectUriInput,
                CreateRedirectUriInput, CreateRoleInput, CreateSamlAttributeMapperInput,
                CreateWebOriginInput, DeleteClientInput, DeletePostLogoutRedirectUriInput,
                DeleteRedirectUriInput, DeleteSamlAttributeMapperInput, DeleteWebOriginInput,
                GetClientInput, GetClientRolesInput, GetClientSamlConfigInput, GetClientsInput,
                GetPostLogoutRedirectUrisInput, GetRedirectUrisInput, GetSamlAttributeMappersInput,
                GetWebOriginsInput, SetClientSamlConfigInput, UpdateClientInput,
                UpdatePostLogoutRedirectUriInput, UpdateRedirectUriInput,
                redirect_uri::RedirectUri,
                saml::{ClientSamlConfig, SamlAttributeMapper},
                web_origin::{Origin, WebOrigin},
            },
            ports::{ClientService, WebOriginResolver},
        },
        common::entities::app_errors::CoreError,
        role::entities::Role,
    },
};

impl ClientService for ApplicationService {
    async fn create_client(
        &self,
        identity: Identity,
        input: CreateClientInput,
    ) -> Result<Client, CoreError> {
        self.client_service.create_client(identity, input).await
    }

    async fn create_redirect_uri(
        &self,
        identity: Identity,
        input: CreateRedirectUriInput,
    ) -> Result<RedirectUri, CoreError> {
        self.client_service
            .create_redirect_uri(identity, input)
            .await
    }

    async fn create_web_origin(
        &self,
        identity: Identity,
        input: CreateWebOriginInput,
    ) -> Result<WebOrigin, CoreError> {
        self.client_service.create_web_origin(identity, input).await
    }

    async fn get_web_origins(
        &self,
        identity: Identity,
        input: GetWebOriginsInput,
    ) -> Result<Vec<WebOrigin>, CoreError> {
        self.client_service.get_web_origins(identity, input).await
    }

    async fn delete_web_origin(
        &self,
        identity: Identity,
        input: DeleteWebOriginInput,
    ) -> Result<(), CoreError> {
        self.client_service.delete_web_origin(identity, input).await
    }

    async fn get_client_saml_config(
        &self,
        identity: Identity,
        input: GetClientSamlConfigInput,
    ) -> Result<ClientSamlConfig, CoreError> {
        self.client_service
            .get_client_saml_config(identity, input)
            .await
    }

    async fn set_client_saml_config(
        &self,
        identity: Identity,
        input: SetClientSamlConfigInput,
    ) -> Result<ClientSamlConfig, CoreError> {
        self.client_service
            .set_client_saml_config(identity, input)
            .await
    }

    async fn create_saml_attribute_mapper(
        &self,
        identity: Identity,
        input: CreateSamlAttributeMapperInput,
    ) -> Result<SamlAttributeMapper, CoreError> {
        self.client_service
            .create_saml_attribute_mapper(identity, input)
            .await
    }

    async fn get_saml_attribute_mappers(
        &self,
        identity: Identity,
        input: GetSamlAttributeMappersInput,
    ) -> Result<Vec<SamlAttributeMapper>, CoreError> {
        self.client_service
            .get_saml_attribute_mappers(identity, input)
            .await
    }

    async fn delete_saml_attribute_mapper(
        &self,
        identity: Identity,
        input: DeleteSamlAttributeMapperInput,
    ) -> Result<(), CoreError> {
        self.client_service
            .delete_saml_attribute_mapper(identity, input)
            .await
    }

    async fn create_post_logout_redirect_uri(
        &self,
        identity: Identity,
        input: CreatePostLogoutRedirectUriInput,
    ) -> Result<RedirectUri, CoreError> {
        self.client_service
            .create_post_logout_redirect_uri(identity, input)
            .await
    }

    async fn create_role(
        &self,
        identity: Identity,
        input: CreateRoleInput,
    ) -> Result<Role, CoreError> {
        self.client_service.create_role(identity, input).await
    }

    async fn delete_client(
        &self,
        identity: Identity,
        input: DeleteClientInput,
    ) -> Result<(), CoreError> {
        self.client_service.delete_client(identity, input).await
    }

    async fn delete_redirect_uri(
        &self,
        identity: Identity,
        input: DeleteRedirectUriInput,
    ) -> Result<(), CoreError> {
        self.client_service
            .delete_redirect_uri(identity, input)
            .await
    }

    async fn delete_post_logout_redirect_uri(
        &self,
        identity: Identity,
        input: DeletePostLogoutRedirectUriInput,
    ) -> Result<(), CoreError> {
        self.client_service
            .delete_post_logout_redirect_uri(identity, input)
            .await
    }

    async fn get_client_by_id(
        &self,
        identity: Identity,
        input: GetClientInput,
    ) -> Result<Client, CoreError> {
        self.client_service.get_client_by_id(identity, input).await
    }

    async fn reveal_client_secret(
        &self,
        identity: Identity,
        input: GetClientInput,
    ) -> Result<Option<String>, CoreError> {
        self.client_service
            .reveal_client_secret(identity, input)
            .await
    }

    async fn get_client_roles(
        &self,
        identity: Identity,
        input: GetClientRolesInput,
    ) -> Result<Vec<Role>, CoreError> {
        self.client_service.get_client_roles(identity, input).await
    }

    async fn get_clients(
        &self,
        identity: Identity,
        input: GetClientsInput,
    ) -> Result<Vec<Client>, CoreError> {
        self.client_service.get_clients(identity, input).await
    }

    async fn get_redirect_uris(
        &self,
        identity: Identity,
        input: GetRedirectUrisInput,
    ) -> Result<Vec<RedirectUri>, CoreError> {
        self.client_service.get_redirect_uris(identity, input).await
    }

    async fn get_post_logout_redirect_uris(
        &self,
        identity: Identity,
        input: GetPostLogoutRedirectUrisInput,
    ) -> Result<Vec<RedirectUri>, CoreError> {
        self.client_service
            .get_post_logout_redirect_uris(identity, input)
            .await
    }

    async fn update_client(
        &self,
        identity: Identity,
        input: UpdateClientInput,
    ) -> Result<Client, CoreError> {
        self.client_service.update_client(identity, input).await
    }

    async fn update_redirect_uri(
        &self,
        identity: Identity,
        input: UpdateRedirectUriInput,
    ) -> Result<RedirectUri, CoreError> {
        self.client_service
            .update_redirect_uri(identity, input)
            .await
    }

    async fn update_post_logout_redirect_uri(
        &self,
        identity: Identity,
        input: UpdatePostLogoutRedirectUriInput,
    ) -> Result<RedirectUri, CoreError> {
        self.client_service
            .update_post_logout_redirect_uri(identity, input)
            .await
    }
}

impl WebOriginResolver for ApplicationService {
    async fn resolve_realm_origins(
        &self,
        realm_name: String,
    ) -> Result<HashSet<Origin>, CoreError> {
        self.client_service.resolve_realm_origins(realm_name).await
    }
}
