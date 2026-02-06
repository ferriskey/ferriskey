pub use ferriskey_domain::role::ports::{RolePolicy, RoleRepository, RoleService};

#[cfg(test)]
pub use mocks::MockRoleRepository;

#[cfg(test)]
mod mocks {
    use mockall::mock;
    use uuid::Uuid;

    use crate::domain::common::entities::app_errors::CoreError;
    use ferriskey_domain::realm::RealmId;
    use ferriskey_domain::role::{
        entities::Role,
        value_objects::{CreateRoleRequest, UpdateRolePermissionsRequest, UpdateRoleRequest},
    };

    mock! {
        pub RoleRepository {}
        impl ferriskey_domain::role::ports::RoleRepository for RoleRepository {

            fn create(
                &self,
                payload: CreateRoleRequest,
            ) -> impl Future<Output = Result<Role, CoreError>> + Send;
            fn get_by_client_id(
                &self,
                client_id: Uuid,
            ) -> impl Future<Output = Result<Vec<Role>, CoreError>> + Send;
            fn get_by_id(
                &self,
                id: Uuid,
            ) -> impl Future<Output = Result<Option<Role>, CoreError>> + Send;
            fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;

            fn find_by_realm_id(
                &self,
                realm_id: RealmId,
            ) -> impl Future<Output = Result<Vec<Role>, CoreError>> + Send;
            fn find_by_name(
                &self,
                name: String,
                realm_id: Uuid,
            ) -> impl Future<Output = Result<Option<Role>, CoreError>> + Send;

            fn update_by_id(
                &self,
                id: Uuid,
                payload: UpdateRoleRequest,
            ) -> impl Future<Output = Result<Role, CoreError>> + Send;

            fn update_permissions_by_id(
                &self,
                id: Uuid,
                payload: UpdateRolePermissionsRequest,
            ) -> impl Future<Output = Result<Role, CoreError>> + Send;
        }
    }
}
