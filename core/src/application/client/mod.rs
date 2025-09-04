use crate::{
    application::common::{FerriskeyService, policies::ensure_policy},
    domain::{
        authentication::value_objects::Identity,
        client::{
            entities::{
                Client, CreateClientInput, CreateRedirectUriInput, CreateRoleInput,
                DeleteClientInput, redirect_uri::RedirectUri,
            },
            ports::{ClientPolicy, ClientRepository, ClientService, RedirectUriRepository},
            value_objects::CreateClientRequest,
        },
        common::{entities::app_errors::CoreError, generate_random_string},
        realm::ports::RealmRepository,
        role::{
            entities::Role,
            ports::{RolePolicy, RoleRepository},
            value_objects::CreateRoleRequest,
        },
    },
};

mod policies;
pub mod use_cases;

impl ClientService for FerriskeyService {
    async fn create_client(
        &self,
        identity: Identity,
        input: CreateClientInput,
    ) -> Result<Client, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_create_client(identity, realm).await,
            "insufficient permissions",
        )?;

        let secret = (!input.public_client).then(generate_random_string);

        let client = self
            .client_repository
            .create_client(CreateClientRequest {
                realm_id: realm_id,
                name: input.name,
                client_id: input.client_id,
                secret,
                enabled: input.enabled,
                protocol: input.protocol,
                public_client: input.public_client,
                service_account_enabled: input.service_account_enabled,
                direct_access_grants_enabled: input.direct_access_grants_enabled,
                client_type: input.client_type,
            })
            .await
            .map_err(|_| CoreError::CreateClientError)?;

        // @TODO: Implement webhook notifier call

        Ok(client)
    }

    async fn create_redirect_uri(
        &self,
        identity: Identity,
        input: CreateRedirectUriInput,
    ) -> Result<RedirectUri, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_create_client(identity, realm).await,
            "insufficient permissions",
        )?;

        let redirect_uri = self
            .redirect_uri_repository
            .create_redirect_uri(input.client_id, input.payload.value, input.payload.enabled)
            .await
            .map_err(|_| CoreError::InvalidRedirectUri)?;

        // @TODO: Implement webhook notifier call

        Ok(redirect_uri)
    }

    async fn create_role(
        &self,
        identity: Identity,
        input: CreateRoleInput,
    ) -> Result<Role, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_create_role(identity, realm).await,
            "insufficient permissions",
        )?;

        let role = self
            .role_repository
            .create(CreateRoleRequest {
                client_id: Some(input.client_id),
                description: input.description,
                name: input.name,
                permissions: input.permissions,
                realm_id,
            })
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(role)
    }

    async fn delete_client(
        &self,
        identity: Identity,
        input: DeleteClientInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_delete_client(identity, realm).await,
            "insufficient permissions",
        )?;

        self.client_repository
            .delete_by_id(input.client_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        // @TODO: Implement webhook notifier
        Ok(())
    }

    async fn delete_redirect_uri(&self) -> Result<(), CoreError> {
        todo!()
    }

    async fn get_client_by_id(&self, id: uuid::Uuid) -> Result<(), CoreError> {
        todo!()
    }

    async fn get_client_roles(&self) -> Result<(), CoreError> {
        todo!()
    }

    async fn get_clients_by_realm_id(
        &self,
        realm_id: uuid::Uuid,
    ) -> Result<Vec<Client>, CoreError> {
        todo!()
    }

    async fn get_redirect_uris(&self, client_id: uuid::Uuid) -> Result<(), CoreError> {
        todo!()
    }

    async fn update_client(&self) -> Result<(), CoreError> {
        todo!()
    }

    async fn update_redirect_uri(&self) -> Result<(), CoreError> {
        todo!()
    }
}
