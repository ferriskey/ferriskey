use uuid::Uuid;

use crate::{
    domain::{
        realm::ports::realm_repository::RealmRepository,
        role::{entities::models::Role, ports::RoleRepository},
        user::{
            entities::error::UserError,
            ports::{user_repository::UserRepository, user_role_service::UserRoleService},
        },
    },
    infrastructure::repositories::{
        realm_repository::PostgresRealmRepository, role_repository::PostgresRoleRepository,
        user_repository::PostgresUserRepository,
    },
};

pub type DefaultUserRoleService =
    UserRoleServiceImpl<PostgresUserRepository, PostgresRoleRepository, PostgresRealmRepository>;

#[derive(Debug, Clone)]
pub struct UserRoleServiceImpl<U, R, RM>
where
    U: UserRepository,
    R: RoleRepository,
    RM: RealmRepository,
{
    pub user_repository: U,
    pub role_repository: R,
    pub realm_repository: RM,
}

impl<U, R, RM> UserRoleServiceImpl<U, R, RM>
where
    U: UserRepository,
    R: RoleRepository,
    RM: RealmRepository,
{
    pub fn new(user_repository: U, role_repository: R, realm_repository: RM) -> Self {
        UserRoleServiceImpl {
            user_repository,
            role_repository,
            realm_repository,
        }
    }
}

impl<U, R, RM> UserRoleService for UserRoleServiceImpl<U, R, RM>
where
    U: UserRepository,
    R: RoleRepository,
    RM: RealmRepository,
{
    async fn assign_role(
        &self,
        realm_name: String,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), UserError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?
            .ok_or(UserError::InternalServerError)?;

        let role = self
            .role_repository
            .get_by_id(role_id)
            .await
            .map_err(|_| UserError::InternalServerError)?
            .ok_or(UserError::InternalServerError)?;

        let user = self.user_repository.get_by_id(user_id).await?;

        if user.realm_id != realm.id || role.realm_id != realm.id {
            return Err(UserError::InternalServerError);
        }

        self.user_repository
            .assign_role_to_user(user.id, role.id)
            .await
    }

    async fn get_user_roles(&self, _user_id: Uuid) -> Result<Vec<Role>, UserError> {
        todo!()
    }

    async fn has_role(&self, _user_id: Uuid, _role_id: Uuid) -> Result<bool, UserError> {
        todo!()
    }

    async fn revoke_role(&self, _user_id: Uuid, _role_id: Uuid) -> Result<(), UserError> {
        todo!()
    }
}
