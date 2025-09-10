use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    realm::entities::Realm,
    role::{
        entities::{GetUserRolesInput, Role, UpdateRoleInput},
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
    ) -> impl Future<Output = Result<Role, CoreError>> + Send;
    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, CoreError>> + Send;
    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<Option<Role>, CoreError>> + Send;
    fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn find_by_realm_id(
        &self,
        realm_id: Uuid,
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
