use uuid::Uuid;

use crate::{
    domain::role::{
        entities::{Role, RoleError},
        ports::RoleRepository,
        value_objects::{CreateRoleRequest, UpdateRolePermissionsRequest, UpdateRoleRequest},
    },
    infrastructure::role::repositories::role_postgres_repository::PostgresRoleRepository,
};

pub mod role_postgres_repository;

#[cfg(test)]
use crate::domain::role::ports::test::MockRoleRepository;

#[derive(Clone)]
pub enum RoleRepoAny {
    Postgres(PostgresRoleRepository),
    #[cfg(test)]
    Mock(MockRoleRepository),
}

impl RoleRepository for RoleRepoAny {
    async fn create(&self, payload: CreateRoleRequest) -> Result<Role, RoleError> {
        match self {
            RoleRepoAny::Postgres(repo) => repo.create(payload).await,
            #[cfg(test)]
            RoleRepoAny::Mock(m) => m.create(payload).await,
        }
    }

    async fn get_by_client_id(&self, client_id: Uuid) -> Result<Vec<Role>, RoleError> {
        match self {
            RoleRepoAny::Postgres(repo) => repo.get_by_client_id(client_id).await,
            #[cfg(test)]
            RoleRepoAny::Mock(m) => m.get_by_client_id(client_id).await,
        }
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Role>, RoleError> {
        match self {
            RoleRepoAny::Postgres(repo) => repo.get_by_id(id).await,
            #[cfg(test)]
            RoleRepoAny::Mock(m) => m.get_by_id(id).await,
        }
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), RoleError> {
        match self {
            RoleRepoAny::Postgres(repo) => repo.delete_by_id(id).await,
            #[cfg(test)]
            RoleRepoAny::Mock(m) => m.delete_by_id(id).await,
        }
    }

    async fn find_by_realm_id(&self, realm_id: Uuid) -> Result<Vec<Role>, RoleError> {
        match self {
            RoleRepoAny::Postgres(repo) => repo.find_by_realm_id(realm_id).await,
            #[cfg(test)]
            RoleRepoAny::Mock(m) => m.find_by_realm_id(realm_id).await,
        }
    }

    async fn find_by_name(&self, name: String, realm_id: Uuid) -> Result<Option<Role>, RoleError> {
        match self {
            RoleRepoAny::Postgres(repo) => repo.find_by_name(name, realm_id).await,
            #[cfg(test)]
            RoleRepoAny::Mock(m) => m.find_by_name(name, realm_id).await,
        }
    }

    async fn update_by_id(&self, id: Uuid, payload: UpdateRoleRequest) -> Result<Role, RoleError> {
        match self {
            RoleRepoAny::Postgres(repo) => repo.update_by_id(id, payload).await,
            #[cfg(test)]
            RoleRepoAny::Mock(m) => m.update_by_id(id, payload).await,
        }
    }

    async fn update_permissions_by_id(
        &self,
        id: Uuid,
        payload: UpdateRolePermissionsRequest,
    ) -> Result<Role, RoleError> {
        match self {
            RoleRepoAny::Postgres(repo) => repo.update_permissions_by_id(id, payload).await,
            #[cfg(test)]
            RoleRepoAny::Mock(m) => m.update_permissions_by_id(id, payload).await,
        }
    }
}
