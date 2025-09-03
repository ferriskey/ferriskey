use crate::{
    application::common::FerriskeyService,
    domain::{
        authentication::value_objects::Identity,
        client::{entities::Client, ports::ClientService, value_objects::CreateClientRequest},
        common::entities::app_errors::CoreError,
    },
};

mod policies;
pub mod use_cases;

impl ClientService for FerriskeyService {
    async fn create_client(
        &self,
        identity: Identity,
        input: CreateClientRequest,
    ) -> Result<Client, CoreError> {
        todo!()
    }

    async fn create_redirect_uri(&self) -> Result<(), CoreError> {
        todo!()
    }

    async fn create_role(&self) -> Result<(), CoreError> {
        todo!()
    }

    async fn delete_client(&self) -> Result<(), CoreError> {
        todo!()
    }

    async fn delete_redirect_uri(&self) -> Result<(), CoreError> {
        todo!()
    }

    async fn get_client_by_id(&self, id: uuid::Uuid) -> Result<(), CoreError> {
        todo!()
    }

    async fn get_client_roles(&self) -> Result<(), CoreError> {
        todo!()
    }

    async fn get_clients_by_realm_id(
        &self,
        realm_id: uuid::Uuid,
    ) -> Result<Vec<Client>, CoreError> {
        todo!()
    }

    async fn get_redirect_uris(&self, client_id: uuid::Uuid) -> Result<(), CoreError> {
        todo!()
    }

    async fn update_client(&self) -> Result<(), CoreError> {
        todo!()
    }

    async fn update_redirect_uri(&self) -> Result<(), CoreError> {
        todo!()
    }
}
