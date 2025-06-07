use crate::domain::client::entities::{dto::CreateClientDto, error::ClientError, model::Client};
use std::future::Future;
use uuid::Uuid;

pub trait ClientRepository: Clone + Send + Sync + 'static {
    fn create_client(
        &self,
        data: CreateClientDto,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<Client, ClientError>> + Send;
    fn get_by_realm_id(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Client>, ClientError>> + Send;
    fn delete_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<(), ClientError>> + Send;
}
