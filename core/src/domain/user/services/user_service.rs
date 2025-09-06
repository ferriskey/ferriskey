use std::collections::HashSet;

use uuid::Uuid;

use crate::domain::{
    realm::{entities::Realm, ports::RealmRepository},
    role::entities::{Role, permission::Permissions},
    user::{
        entities::{RequiredAction, User, UserError},
        ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository, UserService},
        value_objects::{CreateUserRequest, UpdateUserRequest},
    },
};

#[derive(Debug, Clone)]
pub struct UserServiceImpl<U, R, UR, URA>
where
    U: UserRepository,
    R: RealmRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
{
    pub user_repository: U,
    pub realm_repository: R,
    pub user_role_repository: UR,
    pub user_required_action_repository: URA,
}

impl<U, R, UR, URA> UserServiceImpl<U, R, UR, URA>
where
    U: UserRepository,
    R: RealmRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
{
    pub fn new(
        user_repository: U,
        realm_repository: R,
        user_role_repository: UR,
        user_required_action_repository: URA,
    ) -> Self {
        Self {
            user_repository,
            realm_repository,
            user_role_repository,
            user_required_action_repository,
        }
    }
}

impl<U, R, UR, URA> UserService for UserServiceImpl<U, R, UR, URA>
where
    U: UserRepository,
    R: RealmRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
{
    async fn create_user(&self, request: CreateUserRequest) -> Result<User, UserError> {
        self.user_repository.create_user(request).await
    }

    async fn get_by_username(&self, username: String, realm_id: Uuid) -> Result<User, UserError> {
        self.user_repository
            .get_by_username(username, realm_id)
            .await
    }

    async fn get_by_client_id(&self, client_id: Uuid) -> Result<User, UserError> {
        self.user_repository.get_by_client_id(client_id).await
    }

    async fn get_by_id(&self, id: uuid::Uuid) -> Result<User, UserError> {
        self.user_repository.get_by_id(id).await
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<Role>, UserError> {
        let roles = self.user_role_repository.get_user_roles(user_id).await?;
        Ok(roles)
    }

    async fn get_user_realms(
        &self,
        user: User,
        realm_name: String,
    ) -> Result<Vec<Realm>, UserError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?
            .ok_or(UserError::InternalServerError)?;

        if realm.name != "master" {
            return Ok(vec![realm.clone()]);
        }

        let user_roles = self.user_role_repository.get_user_roles(user.id).await?;

        let realms = self
            .realm_repository
            .fetch_realm()
            .await
            .map_err(|_| UserError::InternalServerError)?;

        let mut user_realms: Vec<Realm> = Vec::new();

        for realm in realms {
            let client_name = format!("{}-realm", realm.name);

            let client_roles = user_roles
                .iter()
                .filter(|role| role.client.is_some())
                .filter(|role| role.client.as_ref().unwrap().name == client_name)
                .collect::<Vec<_>>();

            let mut permissions = HashSet::new();

            for role in client_roles {
                let role_permissions = role
                    .permissions
                    .iter()
                    .filter_map(|perm_str| Permissions::from_name(perm_str))
                    .collect::<HashSet<Permissions>>();

                permissions.extend(role_permissions);
            }

            let has_access = Permissions::has_one_of_permissions(
                &permissions.iter().cloned().collect::<Vec<Permissions>>(),
                &[
                    Permissions::QueryRealms,
                    Permissions::ManageRealm,
                    Permissions::ViewRealm,
                ],
            );

            if has_access {
                user_realms.push(realm.clone());
            }
        }

        Ok(user_realms)
    }

    async fn find_by_realm_id(&self, realm_id: Uuid) -> Result<Vec<User>, UserError> {
        self.user_repository.find_by_realm_id(realm_id).await
    }

    async fn bulk_delete_user(&self, ids: Vec<Uuid>) -> Result<u64, UserError> {
        self.user_repository.bulk_delete_user(ids).await
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<u64, UserError> {
        self.user_repository.delete_user(user_id).await
    }

    async fn update_user(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<User, UserError> {
        let required_actions = request.required_actions.clone();
        let user = self.user_repository.update_user(user_id, request).await?;

        if let Some(required_actions) = required_actions {
            self.user_required_action_repository
                .clear_required_actions(user_id)
                .await
                .map_err(|_| UserError::InternalServerError)?;

            for action in required_actions {
                let required_action: RequiredAction =
                    RequiredAction::try_from(action).map_err(|_| UserError::InternalServerError)?;
                self.user_required_action_repository
                    .add_required_action(user_id, required_action)
                    .await
                    .map_err(|_| UserError::InternalServerError)?;
            }
        }

        Ok(user)
    }

    async fn remove_required_action(
        &self,
        user_id: Uuid,
        required_action: RequiredAction,
    ) -> Result<(), UserError> {
        self.user_required_action_repository
            .remove_required_action(user_id, required_action)
            .await
            .map_err(|_| UserError::InternalServerError)
    }

    async fn reset_password(
        &self,
        identity: crate::domain::authentication::value_objects::Identity,
        input: crate::domain::user::entities::ResetPasswordInput,
    ) -> Result<(), crate::domain::common::entities::app_errors::CoreError> {
        unimplemented!()
    }
}
