use std::sync::Arc;

use crate::domain::realm::ports::RealmService;

use super::{
    entities::{
        error::ClientError,
        model::{Client, CreateClientSchema},
    },
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
        _schema: CreateClientSchema,
        realm_name: String,
    ) -> Result<Client, ClientError> {
        let _realm = self
            .realm_service
            .get_by_name(realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;
        todo!("Implement create_client")
    }
}
