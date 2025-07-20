use uuid::Uuid;

use crate::{
    domain::role::entities::{UpdateRoleDto, UpdateRolePermissionsDto},
    infrastructure::repositories::role_repository::PostgresRoleRepository,
};

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

    async fn delete_by_id(&self, _id: Uuid) -> Result<(), RoleError> {
        self.role_repository
            .delete_by_id(_id)
            .await
            .map_err(|_| RoleError::NotFound)
    }

    async fn get_by_client_id(&self, client_id: Uuid) -> Result<Vec<Role>, RoleError> {
        self.role_repository
            .get_by_client_id(client_id)
            .await
            .map_err(|_| RoleError::NotFound)
    }

    async fn get_by_client_id_text(
        &self,
        _client_id: String,
        _realm_id: Uuid,
    ) -> Result<Vec<Role>, RoleError> {
        todo!("get role by client id text");
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Role, RoleError> {
        self.role_repository
            .get_by_id(id)
            .await
            .and_then(|role| role.ok_or(RoleError::NotFound))
    }

    async fn get_by_realm_id(&self, realm_id: Uuid) -> Result<Vec<Role>, RoleError> {
        self.role_repository.find_by_realm_id(realm_id).await
    }

    async fn find_by_name(&self, name: String, realm_id: Uuid) -> Result<Role, RoleError> {
        self.role_repository
            .find_by_name(name, realm_id)
            .await?
            .ok_or(RoleError::NotFound)
    }

    async fn update_by_id(&self, id: Uuid, payload: UpdateRoleDto) -> Result<Role, RoleError> {
        self.role_repository
            .update_by_id(id, payload)
            .await
            .map_err(|_| RoleError::NotFound)
    }

    async fn update_permissions_by_id(
        &self,
        id: Uuid,
        payload: UpdateRolePermissionsDto,
    ) -> Result<Role, RoleError> {
        self.role_repository
            .update_permissions_by_id(id, payload)
            .await
            .map_err(|_| RoleError::NotFound)
    }
}
