use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    realm::entities::Realm,
    role::{
        entities::{GetUserRolesInput, Role, RoleError, UpdateRoleInput},
        value_objects::{CreateRoleRequest, UpdateRolePermissionsRequest, UpdateRoleRequest},
    },
};

pub trait RoleService: Send + Sync + Clone {
    fn delete_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn get_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: Uuid,
    ) -> impl Future<Output = Result<Role, CoreError>> + Send;
    fn get_roles(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> impl Future<Output = Result<Vec<Role>, CoreError>> + Send;
    fn update_role_permissions(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: Uuid,
        permissions: Vec<String>,
    ) -> impl Future<Output = Result<Role, CoreError>> + Send;
    fn update_role(
        &self,
        identity: Identity,
        input: UpdateRoleInput,
    ) -> impl Future<Output = Result<Role, CoreError>> + Send;
    fn get_user_roles(
        &self,
        identity: Identity,
        input: GetUserRolesInput,
    ) -> impl Future<Output = Result<Vec<Role>, CoreError>> + Send;
}

pub trait RolePolicy: Send + Sync + Clone {
    fn can_create_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_view_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_update_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_delete_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub trait RoleRepository: Send + Sync + Clone {
    fn create(
        &self,
        payload: CreateRoleRequest,
    ) -> impl Future<Output = Result<Role, RoleError>> + Send;
    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, RoleError>> + Send;
    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<Option<Role>, RoleError>> + Send;
    fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), RoleError>> + Send;

    fn find_by_realm_id(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, RoleError>> + Send;
    fn find_by_name(
        &self,
        name: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<Role>, RoleError>> + Send;

    fn update_by_id(
        &self,
        id: Uuid,
        payload: UpdateRoleRequest,
    ) -> impl Future<Output = Result<Role, RoleError>> + Send;

    fn update_permissions_by_id(
        &self,
        id: Uuid,
        payload: UpdateRolePermissionsRequest,
    ) -> impl Future<Output = Result<Role, RoleError>> + Send;
}

#[cfg(test)]
pub mod test {
    use super::*;
    use mockall::mock;

    mock! {
        pub RoleService {}
        impl Clone for RoleService { fn clone(&self) -> Self; }
        impl RoleService for RoleService {
            fn delete_role(&self, identity: Identity, realm_name: String, role_id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
            fn get_role(&self, identity: Identity, realm_name: String, role_id: Uuid) -> impl Future<Output = Result<Role, CoreError>> + Send;
            fn get_roles(&self, identity: Identity, realm_name: String) -> impl Future<Output = Result<Vec<Role>, CoreError>> + Send;
            fn update_role_permissions(&self, identity: Identity, realm_name: String, role_id: Uuid, permissions: Vec<String>) -> impl Future<Output = Result<Role, CoreError>> + Send;
            fn update_role(&self, identity: Identity, input: UpdateRoleInput) -> impl Future<Output = Result<Role, CoreError>> + Send;
            fn get_user_roles(&self, identity: Identity, input: GetUserRolesInput) -> impl Future<Output = Result<Vec<Role>, CoreError>> + Send;
        }
    }
    pub fn get_mock_role_service_with_clone_expectations() -> MockRoleService {
        let mut mock = MockRoleService::new();
        mock.expect_clone()
            .returning(|| get_mock_role_service_with_clone_expectations());
        mock
    }
    mock! {
        pub RoleRepository {}
        impl Clone for RoleRepository { fn clone(&self) -> Self; }
        impl RoleRepository for RoleRepository {
            fn create(&self, payload: CreateRoleRequest) -> impl Future<Output = Result<Role, RoleError>> + Send;
            fn get_by_client_id(&self, client_id: Uuid) -> impl Future<Output = Result<Vec<Role>, RoleError>> + Send;
            fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<Option<Role>, RoleError>> + Send;
            fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), RoleError>> + Send;
            fn find_by_realm_id(&self, realm_id: Uuid) -> impl Future<Output = Result<Vec<Role>, RoleError>> + Send;
            fn find_by_name(&self, name: String, realm_id: Uuid) -> impl Future<Output = Result<Option<Role>, RoleError>> + Send;
            fn update_by_id(&self, id: Uuid, payload: UpdateRoleRequest) -> impl Future<Output = Result<Role, RoleError>> + Send;
            fn update_permissions_by_id(&self, id: Uuid, payload: UpdateRolePermissionsRequest) -> impl Future<Output = Result<Role, RoleError>> + Send;
        }
    }
    pub fn get_mock_role_repository_with_clone_expectations() -> MockRoleRepository {
        let mut mock = MockRoleRepository::new();
        mock.expect_clone()
            .returning(|| get_mock_role_repository_with_clone_expectations());
        mock
    }
    mock! {
        pub RolePolicy {}
        impl Clone for RolePolicy { fn clone(&self) -> Self; }
        impl RolePolicy for RolePolicy {
            fn can_create_role(&self, identity: Identity, target_realm: Realm) -> impl Future<Output = Result<bool, CoreError>> + Send;
            fn can_view_role(&self, identity: Identity, target_realm: Realm) -> impl Future<Output = Result<bool, CoreError>> + Send;
            fn can_update_role(&self, identity: Identity, target_realm: Realm) -> impl Future<Output = Result<bool, CoreError>> + Send;
            fn can_delete_role(&self, identity: Identity, target_realm: Realm) -> impl Future<Output = Result<bool, CoreError>> + Send;
        }
    }
    pub fn get_mock_role_policy_with_clone_expectations() -> MockRolePolicy {
        let mut mock = MockRolePolicy::new();
        mock.expect_clone()
            .returning(|| get_mock_role_policy_with_clone_expectations());
        mock
    }
}
