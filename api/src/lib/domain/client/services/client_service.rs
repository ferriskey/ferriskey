use crate::domain::client::entities::dto::{CreateClientDto, UpdateClientDto};
use crate::domain::client::entities::{error::ClientError, model::Client};
use crate::domain::client::ports::client_repository::ClientRepository;
use crate::domain::client::ports::client_service::ClientService;
use crate::domain::realm::ports::realm_service::RealmService;
use crate::domain::realm::services::realm_service::DefaultRealmService;
use crate::domain::user::dtos::user_dto::CreateUserDto;
use crate::domain::user::ports::user_repository::UserRepository;
use crate::infrastructure::repositories::client_repository::PostgresClientRepository;
use crate::infrastructure::user::repository::PostgresUserRepository;
use uuid::Uuid;

pub type DefaultClientService = ClientServiceImpl<PostgresClientRepository, PostgresUserRepository>;

#[derive(Debug, Clone)]
pub struct ClientServiceImpl<C, U>
where
    C: ClientRepository,
{
    pub client_repository: C,
    pub user_repository: U,
    pub realm_service: DefaultRealmService,
}

impl<C, U> ClientServiceImpl<C, U>
where
    C: ClientRepository,
    U: UserRepository,
{
    pub fn new(
        client_repository: C,
        user_repository: U,
        realm_service: DefaultRealmService,
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
        schema: CreateClientDto,
        realm_name: String,
    ) -> Result<Client, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let client = self
            .client_repository
            .create_client(schema.clone())
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        if schema.service_account_enabled {
            self.user_repository
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
        }

        Ok(client)
    }

    async fn update_client(
        &self,
        client_id: Uuid,
        realm_name: String,
        schema: UpdateClientDto,
    ) -> Result<Client, ClientError> {
        let _ = self
            .realm_service
            .get_by_name(realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let client = self
            .client_repository
            .update_client(client_id, schema)
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

    async fn delete_by_id(&self, id: Uuid) -> Result<(), ClientError> {
        self.client_repository.delete_by_id(id).await
    }
}
