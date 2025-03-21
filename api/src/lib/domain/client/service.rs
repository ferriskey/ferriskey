use std::sync::Arc;

use rand::Rng;

use crate::{
    application::http::client::validators::CreateClientValidator,
    domain::realm::ports::RealmService,
};

use super::{
    entities::{error::ClientError, model::Client},
    ports::{ClientRepository, ClientService},
};

#[derive(Debug, Clone)]
pub struct ClientServiceImpl<C, R>
where
    C: ClientRepository,
    R: RealmService,
{
    pub client_repository: C,
    pub realm_service: Arc<R>,
}

impl<C, R> ClientServiceImpl<C, R>
where
    C: ClientRepository,
    R: RealmService,
{
    pub fn new(client_repository: C, realm_service: Arc<R>) -> Self {
        Self {
            client_repository,
            realm_service,
        }
    }
}

impl<C, R> ClientService for ClientServiceImpl<C, R>
where
    C: ClientRepository,
    R: RealmService,
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
        let secret = rand::rng()
            .sample_iter(&rand::distr::Alphanumeric)
            .take(15)
            .map(char::from)
            .collect::<String>();

        self.client_repository
            .create_client(
                realm.id,
                schema.name,
                schema.client_id,
                secret,
                schema.enabled,
                schema.protocol,
                schema.public_client,
                schema.service_account_enabled,
                schema.client_type,
            )
            .await
    }
}
