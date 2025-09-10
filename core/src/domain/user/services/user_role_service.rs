use uuid::Uuid;

use crate::domain::{
    common::entities::app_errors::CoreError,
    realm::ports::RealmRepository,
    role::{entities::Role, ports::RoleRepository},
    user::ports::{UserRepository, UserRoleRepository, UserRoleService},
};

#[derive(Clone)]
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
        Self {
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
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        let role = self
            .role_repository
            .get_by_id(role_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        let user = self.user_repository.get_by_id(user_id).await?;

        if user.realm_id != realm.id || role.realm_id != realm.id {
            return Err(CoreError::InternalServerError);
        }

        self.user_role_repository
            .assign_role(user.id, role.id)
            .await
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, CoreError> {
        self.user_role_repository.get_user_roles(user_id).await
    }

    async fn has_role(&self, _user_id: Uuid, _role_id: Uuid) -> Result<bool, CoreError> {
        unimplemented!("has_role method is not implemented yet");
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<(), CoreError> {
        self.user_role_repository
            .revoke_role(user_id, role_id)
            .await
    }
}
