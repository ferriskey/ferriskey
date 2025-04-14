use crate::domain::client::entities::{error::ClientError, model::Client};
use std::future::Future;
use uuid::Uuid;

pub trait ClientRepository: Clone + Send + Sync + 'static {
    fn create_client(
        &self,
        realm_id: Uuid,
        name: String,
        client_id: String,
        secret: Option<String>,
        enabled: bool,
        protocol: String,
        public_client: bool,
        service_account_enabled: bool,
        client_type: String,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Client, ClientError>> + Send;
}
