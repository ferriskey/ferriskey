use uuid::Uuid;

use crate::{
    domain::{
        realm::ports::realm_repository::RealmRepository,
        role::{entities::models::Role, ports::RoleRepository},
        user::{
            entities::error::UserError,
            ports::{
                user_repository::UserRepository, user_role_repository::UserRoleRepository,
                user_role_service::UserRoleService,
            },
        },
    },
    infrastructure::{
        repositories::{
            realm_repository::PostgresRealmRepository, role_repository::PostgresRoleRepository,
        },
        user::{
            repositories::user_role_repository::PostgresUserRoleRepository,
            repository::PostgresUserRepository,
        },
    },
};

pub type DefaultUserRoleService = UserRoleServiceImpl<
    PostgresUserRepository,
    PostgresRoleRepository,
    PostgresRealmRepository,
    PostgresUserRoleRepository,
>;

#[derive(Debug, Clone)]
pub struct UserRoleServiceImpl<U, R, RM, UR>
where
    U: UserRepository,
    R: RoleRepository,
    RM: RealmRepository,
    UR: UserRoleRepository,
{
    pub user_repository: U,
    pub role_repository: R,
    pub realm_repository: RM,
    pub user_role_repository: UR,
}

impl<U, R, RM, UR> UserRoleServiceImpl<U, R, RM, UR>
where
    U: UserRepository,
    R: RoleRepository,
    RM: RealmRepository,
    UR: UserRoleRepository,
{
    pub fn new(
        user_repository: U,
        role_repository: R,
        realm_repository: RM,
        user_role_repository: UR,
    ) -> Self {
        UserRoleServiceImpl {
            user_repository,
            role_repository,
            realm_repository,
            user_role_repository,
        }
    }
}

impl<U, R, RM, UR> UserRoleService for UserRoleServiceImpl<U, R, RM, UR>
where
    U: UserRepository,
    R: RoleRepository,
    RM: RealmRepository,
    UR: UserRoleRepository,
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

        self.user_role_repository
            .assign_role(user.id, role.id)
            .await
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, UserError> {
        self.user_role_repository.get_user_roles(user_id).await
    }

    async fn has_role(&self, _user_id: Uuid, _role_id: Uuid) -> Result<bool, UserError> {
        unimplemented!("has_role method is not implemented yet");
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), UserError> {
        self.user_role_repository
            .revoke_role(user_id, role_id)
            .await
    }
}
