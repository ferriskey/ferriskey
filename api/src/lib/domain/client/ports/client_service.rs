use uuid::Uuid;

use crate::application::http::client::validators::CreateClientValidator;
use crate::domain::client::entities::dto::UpdateClientDto;
use crate::domain::client::entities::{error::ClientError, model::Client};

pub trait ClientService: Clone + Send + Sync + 'static {
    fn create_client(
        &self,
        schema: CreateClientValidator,
        realm_name: String,
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

    fn update_client(
        &self,
        client_id: Uuid,
        realm_name: String,
        schema: UpdateClientDto,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), ClientError>> + Send;
}
