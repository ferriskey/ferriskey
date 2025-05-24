use uuid::Uuid;

use crate::{
    domain::{
        role::{entities::models::Role, ports::RoleRepository},
        user::{
            entities::error::UserError,
            ports::{user_repository::UserRepository, user_role_service::UserRoleService},
        },
    },
    infrastructure::repositories::{
        role_repository::PostgresRoleRepository, user_repository::PostgresUserRepository,
    },
};

pub type DefaultUserRoleService =
    UserRoleServiceImpl<PostgresUserRepository, PostgresRoleRepository>;

#[derive(Debug, Clone)]
pub struct UserRoleServiceImpl<U, R>
where
    U: UserRepository,
    R: RoleRepository,
{
    pub user_repository: U,
    pub role_repository: R,
}

impl<U, R> UserRoleServiceImpl<U, R>
where
    U: UserRepository,
    R: RoleRepository,
{
    pub fn new(user_repository: U, role_repository: R) -> Self {
        UserRoleServiceImpl {
            user_repository,
            role_repository,
        }
    }
}

impl<U, R> UserRoleService for UserRoleServiceImpl<U, R>
where
    U: UserRepository,
    R: RoleRepository,
{
    async fn assign_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), UserError> {
        todo!()
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, UserError> {
        todo!()
    }

    async fn has_role(&self, user_id: Uuid, role_id: Uuid) -> Result<bool, UserError> {
        todo!()
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), UserError> {
        todo!()
    }
}
