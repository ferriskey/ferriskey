use std::collections::HashSet;

use uuid::Uuid;

use crate::{
    domain::{
        realm::{entities::realm::Realm, ports::realm_repository::RealmRepository},
        role::entities::{models::Role, permission::Permissions},
        user::{
            dtos::user_dto::CreateUserDto,
            entities::{error::UserError, model::User},
            ports::{user_repository::UserRepository, user_service::UserService},
        },
    },
    infrastructure::repositories::{
        realm_repository::PostgresRealmRepository, user_repository::PostgresUserRepository,
    },
};

pub type DefaultUserService = UserServiceImpl<PostgresUserRepository, PostgresRealmRepository>;

#[derive(Debug, Clone)]
pub struct UserServiceImpl<U, R>
where
    U: UserRepository,
    R: RealmRepository,
{
    pub user_repository: U,
    pub realm_repository: R,
}

impl<U, R> UserServiceImpl<U, R>
where
    U: UserRepository,
    R: RealmRepository,
{
    pub fn new(user_repository: U, realm_repository: R) -> Self {
        Self {
            user_repository,
            realm_repository,
        }
    }
}

impl<U, R> UserService for UserServiceImpl<U, R>
where
    U: UserRepository,
    R: RealmRepository,
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
        let roles = self.user_repository.get_roles_by_user_id(user_id).await?;
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

        if realm.name != "master".to_string() {
            return Ok(vec![realm.clone()]);
        }

        let user_roles = self.user_repository.get_roles_by_user_id(user.id).await?;

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
                let bitfield = role.permissions.clone() as u64;
                let role_permissions = Permissions::from_bitfield(bitfield);

                for permission in role_permissions {
                    permissions.insert(permission);
                }
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
}
