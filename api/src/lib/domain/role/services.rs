use crate::infrastructure::repositories::role_repository::PostgresRoleRepository;

use super::{
    entities::{CreateRoleDto, errors::RoleError, models::Role},
    ports::{RoleRepository, RoleService},
};
pub type DefaultRoleService = RoleServiceImpl<PostgresRoleRepository>;

#[derive(Debug, Clone)]
pub struct RoleServiceImpl<R>
where
    R: RoleRepository,
{
    pub role_repository: R,
}

impl<R> RoleServiceImpl<R>
where
    R: RoleRepository,
{
    pub fn new(role_repository: R) -> Self {
        Self { role_repository }
    }
}

impl<R> RoleService for RoleServiceImpl<R>
where
    R: RoleRepository,
{
    async fn create(&self, payload: CreateRoleDto) -> Result<Role, RoleError> {
        let role = self.role_repository.create(payload).await?;
        Ok(role)
    }

    async fn delete_by_id(&self, _id: uuid::Uuid) -> Result<(), RoleError> {
        todo!("delete role");
    }

    async fn get_by_client_id(&self, _client_id: uuid::Uuid) -> Result<Vec<Role>, RoleError> {
        todo!("get role by client id");
    }

    async fn get_by_client_id_text(
        &self,
        _client_id: String,
        _realm_id: uuid::Uuid,
    ) -> Result<Vec<Role>, RoleError> {
        todo!("get role by client id text");
    }

    async fn get_by_id(&self, _id: uuid::Uuid) -> Result<Role, RoleError> {
        todo!("get role by id");
    }

    async fn get_by_realm_id(&self, realm_id: uuid::Uuid) -> Result<Vec<Role>, RoleError> {
        self.role_repository.find_by_realm_id(realm_id).await
    }

    async fn find_by_name(&self, name: String, realm_id: uuid::Uuid) -> Result<Role, RoleError> {
        self.role_repository
            .find_by_name(name, realm_id)
            .await?
            .ok_or(RoleError::NotFound)
    }
}
