use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    realm::entities::Realm,
    role::{
        entities::{Role, RoleError},
        value_objects::{CreateRoleRequest, UpdateRolePermissionsRequest, UpdateRoleRequest},
    },
};

pub trait RoleService: Send + Sync + Clone {
    #[deprecated]
    fn create(
        &self,
        payload: CreateRoleRequest,
    ) -> impl Future<Output = Result<Role, RoleError>> + Send;
    #[deprecated]
    fn get_by_realm_id(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, RoleError>> + Send;
    #[deprecated]
    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, RoleError>> + Send;
    #[deprecated]
    fn get_by_client_id_text(
        &self,
        client_id: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, RoleError>> + Send;
    #[deprecated]
    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<Role, RoleError>> + Send;
    #[deprecated]
    fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), RoleError>> + Send;
    #[deprecated]
    fn find_by_name(
        &self,
        name: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Role, RoleError>> + Send;
    #[deprecated]
    fn update_by_id(
        &self,
        id: Uuid,
        payload: UpdateRoleRequest,
    ) -> impl Future<Output = Result<Role, RoleError>> + Send;

    #[deprecated]
    fn update_permissions_by_id(
        &self,
        id: Uuid,
        payload: UpdateRolePermissionsRequest,
    ) -> impl Future<Output = Result<Role, RoleError>> + Send;

    fn delete_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
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
