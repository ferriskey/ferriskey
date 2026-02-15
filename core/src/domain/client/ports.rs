pub use ferriskey_domain::client::ports::{
    ClientPolicy, ClientRepository, ClientService, RedirectUriRepository, RedirectUriService,
};
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;

#[cfg(test)]
pub use mocks::{
    MockClientRepository, MockPostLogoutRedirectUriRepository, MockRedirectUriRepository,
};

#[cfg(test)]
mod mocks {
    use mockall::mock;
    use uuid::Uuid;

    use crate::domain::common::entities::app_errors::CoreError;
    use ferriskey_domain::client::{
        entities::{Client, redirect_uri::RedirectUri},
        value_objects::{CreateClientRequest, UpdateClientRequest},
    };
    use ferriskey_domain::realm::RealmId;

    mock! {
        pub ClientRepository {}
        impl ferriskey_domain::client::ports::ClientRepository for ClientRepository {
            fn create_client(
                &self,
                data: CreateClientRequest,
            ) -> impl Future<Output = Result<Client, CoreError>> + Send;

            fn get_by_client_id(
                &self,
                client_id: String,
                realm_id: RealmId,
            ) -> impl Future<Output = Result<Client, CoreError>> + Send;

            fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<Client, CoreError>> + Send;
            fn get_by_realm_id(
                &self,
                realm_id: RealmId,
            ) -> impl Future<Output = Result<Vec<Client>, CoreError>> + Send;

            fn update_client(
                &self,
                client_id: Uuid,
                data: UpdateClientRequest,
            ) -> impl Future<Output = Result<Client, CoreError>> + Send;

            fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
        }
    }

    mock! {
        pub RedirectUriRepository {}
        impl ferriskey_domain::client::ports::RedirectUriRepository for RedirectUriRepository {
            fn create_redirect_uri(
                &self,
                client_id: Uuid,
                value: String,
                enabled: bool,
            ) -> impl Future<Output = Result<RedirectUri, CoreError>> + Send;

            fn get_by_client_id(
                &self,
                client_id: Uuid,
            ) -> impl Future<Output = Result<Vec<RedirectUri>, CoreError>> + Send;

            fn get_enabled_by_client_id(
                &self,
                client_id: Uuid,
            ) -> impl Future<Output = Result<Vec<RedirectUri>, CoreError>> + Send;

            fn update_enabled(
                &self,
                id: Uuid,
                enabled: bool,
            ) -> impl Future<Output = Result<RedirectUri, CoreError>> + Send;

            fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
        }
    }

    mock! {
        pub PostLogoutRedirectUriRepository {}
        impl super::PostLogoutRedirectUriRepository for PostLogoutRedirectUriRepository {
            fn get_enabled_by_client_id(
                &self,
                client_id: Uuid,
            ) -> impl Future<Output = Result<Vec<String>, CoreError>> + Send;
        }
    }
}

pub trait PostLogoutRedirectUriRepository: Send + Sync {
    fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<String>, CoreError>> + Send;
}
