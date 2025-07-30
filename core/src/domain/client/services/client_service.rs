use uuid::Uuid;

use crate::domain::{
    client::{
        entities::{Client, ClientError},
        ports::{ClientRepository, ClientService},
        value_objects::{CreateClientRequest, UpdateClientRequest},
    },
    realm::ports::RealmRepository,
    user::{ports::UserRepository, value_objects::CreateUserRequest},
};

#[derive(Debug, Clone)]
pub struct ClientServiceImpl<C, U, R>
where
    C: ClientRepository,
    U: UserRepository,
    R: RealmRepository,
{
    pub client_repository: C,
    pub user_repository: U,
    pub realm_repository: R,
}

impl<C, U, R> ClientServiceImpl<C, U, R>
where
    C: ClientRepository,
    U: UserRepository,
    R: RealmRepository,
{
    pub fn new(client_repository: C, user_repository: U, realm_repository: R) -> Self {
        Self {
            client_repository,
            user_repository,
            realm_repository,
        }
    }
}

impl<C, U, R> ClientService for ClientServiceImpl<C, U, R>
where
    C: ClientRepository,
    U: UserRepository,
    R: RealmRepository,
{
    async fn create_client(
        &self,
        schema: CreateClientRequest,
        realm_name: String,
    ) -> Result<Client, ClientError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?
            .ok_or(ClientError::InternalServerError)?;

        let client = self
            .client_repository
            .create_client(schema.clone())
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        if schema.service_account_enabled {
            self.user_repository
                .create_user(CreateUserRequest {
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
        schema: UpdateClientRequest,
    ) -> Result<Client, ClientError> {
        let _ = self
            .realm_repository
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
        realm_id: Uuid,
    ) -> Result<Client, ClientError> {
        self.client_repository
            .get_by_client_id(client_id, realm_id)
            .await
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Client, ClientError> {
        self.client_repository
            .get_by_id(id)
            .await
            .map_err(|_| ClientError::NotFound)
    }

    async fn get_by_realm_id(&self, realm_id: Uuid) -> Result<Vec<Client>, ClientError> {
        self.client_repository
            .get_by_realm_id(realm_id)
            .await
            .map_err(|_| ClientError::NotFound)
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), ClientError> {
        self.client_repository.delete_by_id(id).await
    }
}
