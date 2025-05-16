use uuid::Uuid;

use super::entities::{CreateRoleDto, errors::RoleError, models::Role};

pub trait RoleService: Send + Sync {
    fn create(
        &self,
        payload: CreateRoleDto,
    ) -> impl Future<Output = Result<Role, RoleError>> + Send;
    fn get_by_realm_id(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, RoleError>> + Send;
    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, RoleError>> + Send;
    fn get_by_client_id_text(
        &self,
        client_id: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, RoleError>> + Send;
    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<Role, RoleError>> + Send;
    fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), RoleError>> + Send;
}

pub trait RoleRepository: Send + Sync {
    fn create(
        &self,
        payload: CreateRoleDto,
    ) -> impl Future<Output = Result<Role, RoleError>> + Send;
    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, RoleError>> + Send;
    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<Option<Role>, RoleError>> + Send;
    fn delete_by_id(&self, id: Uuid) -> impl Future<Output = Result<(), RoleError>> + Send;
}
