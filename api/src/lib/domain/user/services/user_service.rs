use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    domain::{
        realm::{entities::realm::Realm, ports::realm_repository::RealmRepository},
        role::entities::{models::Role, permission::Permissions},
        user::{
            dtos::user_dto::{CreateUserDto, UpdateUserDto},
            entities::{error::UserError, model::User},
            ports::{
                user_repository::UserRepository, user_role_repository::UserRoleRepository,
                user_service::UserService,
            },
        },
    },
    infrastructure::{
        repositories::realm_repository::PostgresRealmRepository,
        user::{
            repositories::user_role_repository::PostgresUserRoleRepository,
            repository::PostgresUserRepository,
        },
    },
};

pub type DefaultUserService =
    UserServiceImpl<PostgresUserRepository, PostgresRealmRepository, PostgresUserRoleRepository>;

#[derive(Debug, Clone)]
pub struct UserServiceImpl<U, R, UR>
where
    U: UserRepository,
    R: RealmRepository,
    UR: UserRoleRepository,
{
    pub user_repository: U,
    pub realm_repository: R,
    pub user_role_repository: UR,
}

impl<U, R, UR> UserServiceImpl<U, R, UR>
where
    U: UserRepository,
    R: RealmRepository,
    UR: UserRoleRepository,
{
    pub fn new(user_repository: U, realm_repository: R, user_role_repository: UR) -> Self {
        Self {
            user_repository,
            realm_repository,
            user_role_repository,
        }
    }
}

impl<U, R, UR> UserService for UserServiceImpl<U, R, UR>
where
    U: UserRepository,
    R: RealmRepository,
    UR: UserRoleRepository,
{
    async fn create_user(&self, dto: CreateUserDto) -> Result<User, UserError> {
        self.user_repository.create_user(dto).await
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

    async fn update_user(&self, user_id: Uuid, dto: UpdateUserDto) -> Result<User, UserError> {
        self.user_repository.update_user(user_id, dto).await
    }
}
