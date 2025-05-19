use std::sync::Arc;

use crate::application::http::client::validators::CreateClientValidator;
use crate::domain::client::entities::dto::CreateClientDto;
use crate::domain::client::entities::{error::ClientError, model::Client};
use crate::domain::client::ports::client_repository::ClientRepository;
use crate::domain::client::ports::client_service::ClientService;
use crate::domain::realm::ports::realm_service::RealmService;
use crate::domain::realm::services::realm_service::DefaultRealmService;
use crate::domain::user::dtos::user_dto::CreateUserDto;
use crate::domain::user::ports::user_repository::UserRepository;
use crate::domain::utils::generate_random_string;
use crate::infrastructure::repositories::{
    client_repository::PostgresClientRepository, user_repository::PostgresUserRepository,
};

pub type DefaultClientService = ClientServiceImpl<PostgresClientRepository, PostgresUserRepository>;

#[derive(Debug, Clone)]
pub struct ClientServiceImpl<C, U>
where
    C: ClientRepository,
{
    pub client_repository: C,
    pub user_repository: U,
    pub realm_service: Arc<DefaultRealmService>,
}

impl<C, U> ClientServiceImpl<C, U>
where
    C: ClientRepository,
    U: UserRepository,
{
    pub fn new(
        client_repository: C,
        user_repository: U,
        realm_service: Arc<DefaultRealmService>,
    ) -> Self {
        Self {
            client_repository,
            user_repository,
            realm_service,
        }
    }
}

impl<C, U> ClientService for ClientServiceImpl<C, U>
where
    C: ClientRepository,
    U: UserRepository,
{
    async fn create_client(
        &self,
        schema: CreateClientValidator,
        realm_name: String,
    ) -> Result<Client, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let client = self
            .client_repository
            .create_client(CreateClientDto {
                realm_id: realm.id,
                name: schema.name,
                client_id: schema.client_id,
                secret: generate_random_string(),
                enabled: schema.enabled,
                protocol: schema.protocol,
                public_client: schema.public_client,
                service_account_enabled: schema.service_account_enabled,
                client_type: schema.client_type,
            })
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let _ = self
            .user_repository
            .create_user(CreateUserDto {
                realm_id: realm.id,
                client_id: Some(client.id),
                username: format!("service-account-{}", client.name),
                firstname: "".to_string(),
                lastname: "".to_string(),
                email: "".to_string(),
                email_verified: false,
                enabled: true,
            })
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        Ok(client)
    }

    async fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: uuid::Uuid,
    ) -> Result<Client, ClientError> {
        self.client_repository
            .get_by_client_id(client_id, realm_id)
            .await
    }

    async fn get_by_id(&self, id: uuid::Uuid) -> Result<Client, ClientError> {
        self.client_repository
            .get_by_id(id)
            .await
            .map_err(|_| ClientError::NotFound)
    }

    async fn get_by_realm_id(&self, realm_id: uuid::Uuid) -> Result<Vec<Client>, ClientError> {
        self.client_repository
            .get_by_realm_id(realm_id)
            .await
            .map_err(|_| ClientError::NotFound)
    }
}
