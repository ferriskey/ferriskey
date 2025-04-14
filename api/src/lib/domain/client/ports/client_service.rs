use uuid::Uuid;

use crate::application::http::client::validators::CreateClientValidator;
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
}
