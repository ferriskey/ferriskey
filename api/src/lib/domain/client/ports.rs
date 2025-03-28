use uuid::Uuid;

use crate::application::http::client::validators::CreateClientValidator;

use super::entities::{error::ClientError, model::Client};

pub trait ClientService: Clone + Send + Sync + 'static {
    fn create_client(
        &self,
        schema: CreateClientValidator,
        realm_name: String,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;
}

pub trait ClientRepository: Clone + Send + Sync + 'static {
    fn create_client(
        &self,
        realm_id: Uuid,
        name: String,
        client_id: String,
        secret: String,
        enabled: bool,
        protocol: String,
        public_client: bool,
        service_account_enabled: bool,
        client_type: String,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;
}
