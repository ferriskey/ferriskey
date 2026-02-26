pub mod policies;
pub mod services;

pub use ferriskey_aegis::entities;
pub use ferriskey_aegis::ports;
pub use ferriskey_aegis::value_objects;

#[cfg(test)]
pub mod mocks {
    use mockall::mock;
    use uuid::Uuid;

    use crate::domain::common::entities::app_errors::CoreError;
    use ferriskey_aegis::{
        entities::{ClientScope, ClientScopeMapping, ProtocolMapper},
        value_objects::{
            CreateClientScopeRequest, CreateProtocolMapperRequest, UpdateClientScopeRequest,
            UpdateProtocolMapperRequest,
        },
    };
    use ferriskey_domain::realm::RealmId;

    mock! {
        pub ClientScopeRepository {}
        impl ferriskey_aegis::ports::ClientScopeRepository for ClientScopeRepository {
            fn create(
                &self,
                payload: CreateClientScopeRequest,
            ) -> impl Future<Output = Result<ClientScope, CoreError>> + Send;

            fn get_by_id(
                &self,
                id: Uuid,
            ) -> impl Future<Output = Result<Option<ClientScope>, CoreError>> + Send;

            fn find_by_realm_id(
                &self,
                realm_id: RealmId,
            ) -> impl Future<Output = Result<Vec<ClientScope>, CoreError>> + Send;

            fn find_by_name(
                &self,
                name: String,
                realm_id: RealmId,
            ) -> impl Future<Output = Result<Option<ClientScope>, CoreError>> + Send;

            fn update_by_id(
                &self,
                id: Uuid,
                payload: UpdateClientScopeRequest,
            ) -> impl Future<Output = Result<ClientScope, CoreError>> + Send;

            fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
        }
    }

    mock! {
        pub ClientScopeMappingRepository {}
        impl ferriskey_aegis::ports::ClientScopeMappingRepository for ClientScopeMappingRepository {
            fn assign_scope_to_client(
                &self,
                client_id: Uuid,
                scope_id: Uuid,
                is_default: bool,
                is_optional: bool,
            ) -> impl Future<Output = Result<ClientScopeMapping, CoreError>> + Send;

            fn remove_scope_from_client(
                &self,
                client_id: Uuid,
                scope_id: Uuid,
            ) -> impl Future<Output = Result<(), CoreError>> + Send;

            fn get_client_scopes(
                &self,
                client_id: Uuid,
            ) -> impl Future<Output = Result<Vec<ferriskey_aegis::entities::ClientScopeMapping>, CoreError>> + Send;

            fn get_default_scopes(
                &self,
                client_id: Uuid,
            ) -> impl Future<Output = Result<Vec<ClientScope>, CoreError>> + Send;

            fn get_optional_scopes(
                &self,
                client_id: Uuid,
            ) -> impl Future<Output = Result<Vec<ClientScope>, CoreError>> + Send;
        }
    }

    mock! {
        pub ProtocolMapperRepository {}
        impl ferriskey_aegis::ports::ProtocolMapperRepository for ProtocolMapperRepository {
            fn create(
                &self,
                payload: CreateProtocolMapperRequest,
            ) -> impl Future<Output = Result<ProtocolMapper, CoreError>> + Send;

            fn get_by_id(
                &self,
                id: Uuid,
            ) -> impl Future<Output = Result<Option<ProtocolMapper>, CoreError>> + Send;

            fn get_by_scope_id(
                &self,
                scope_id: Uuid,
            ) -> impl Future<Output = Result<Vec<ProtocolMapper>, CoreError>> + Send;

            fn update_by_id(
                &self,
                id: Uuid,
                payload: UpdateProtocolMapperRequest,
            ) -> impl Future<Output = Result<ProtocolMapper, CoreError>> + Send;

            fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
        }
    }
}
