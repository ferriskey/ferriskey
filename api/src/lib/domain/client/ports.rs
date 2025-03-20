use uuid::Uuid;

use super::entities::{
    error::ClientError,
    model::{Client, CreateClientSchema},
};

pub trait ClientService: Clone + Send + Sync + 'static {
    fn create_client(
        &self,
        schema: CreateClientSchema,
        realm_name: String,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;
}

pub trait ClientRepository: Clone + Send + Sync + 'static {
    fn create_client(
        &self,
        realm_id: Uuid,
        name: String,
        client_id: String,
        enabled: bool,
        protocol: String,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;
}
