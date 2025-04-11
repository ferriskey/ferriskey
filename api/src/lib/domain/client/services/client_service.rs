use std::sync::Arc;

use crate::application::http::client::validators::CreateClientValidator;
use crate::domain::client::entities::{error::ClientError, model::Client};
use crate::domain::client::ports::client_repository::ClientRepository;
use crate::domain::client::ports::client_service::ClientService;
use crate::domain::realm::ports::realm_service::RealmService;
use crate::domain::realm::services::realm_service::DefaultRealmService;
use crate::infrastructure::repositories::client_repository::PostgresClientRepository;

pub type DefaultClientService = ClientServiceImpl<PostgresClientRepository>;

#[derive(Debug, Clone)]
pub struct ClientServiceImpl<C>
where
    C: ClientRepository,
{
    pub client_repository: C,
    pub realm_service: Arc<DefaultRealmService>,
}

impl<C> ClientServiceImpl<C>
where
    C: ClientRepository,
{
    pub fn new(client_repository: C, realm_service: Arc<DefaultRealmService>) -> Self {
        Self {
            client_repository,
            realm_service,
        }
    }
}

impl<C> ClientService for ClientServiceImpl<C>
where
    C: ClientRepository,
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

        self.client_repository
            .create_client(
                realm.id,
                schema.name,
                schema.client_id,
                schema.secret,
                schema.enabled,
                schema.protocol,
                schema.public_client,
                schema.service_account_enabled,
                schema.client_type,
            )
            .await
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
}
