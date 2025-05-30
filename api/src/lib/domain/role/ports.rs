use uuid::Uuid;

use super::entities::{CreateRoleDto, errors::RoleError, models::Role};

pub trait RoleService: Send + Sync + Clone {
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
    fn find_by_name(
        &self,
        name: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Role, RoleError>> + Send;
}

pub trait RoleRepository: Send + Sync + Clone {
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

    fn find_by_realm_id(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, RoleError>> + Send;
    fn find_by_name(
        &self,
        name: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<Role>, RoleError>> + Send;
}
