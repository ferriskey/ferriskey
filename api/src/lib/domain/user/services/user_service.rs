use uuid::Uuid;

use crate::{
    domain::user::{
        dtos::user_dto::CreateUserDto,
        entities::{error::UserError, model::User},
        ports::{user_repository::UserRepository, user_service::UserService},
    },
    infrastructure::repositories::user_repository::PostgresUserRepository,
};

pub type DefaultUserService = UserServiceImpl<PostgresUserRepository>;

#[derive(Debug, Clone)]
pub struct UserServiceImpl<U>
where
    U: UserRepository,
{
    pub user_repository: U,
}

impl<U> UserServiceImpl<U>
where
    U: UserRepository,
{
    pub fn new(user_repository: U) -> Self {
        Self { user_repository }
    }
}

impl<U> UserService for UserServiceImpl<U>
where
    U: UserRepository,
{
    async fn create_user(&self, dto: CreateUserDto) -> Result<User, UserError> {
        self.user_repository.create_user(dto).await
    }

    async fn get_by_username(&self, username: String, realm_id: Uuid) -> Result<User, UserError> {
        self.user_repository
            .get_by_username(username, realm_id)
            .await
    }

    async fn get_by_client_id(&self, client_id: Uuid, realm_id: Uuid) -> Result<User, UserError> {
        self.user_repository
            .get_by_client_id(client_id, realm_id)
            .await
    }

    async fn get_by_id(&self, id: uuid::Uuid) -> Result<User, UserError> {
        self.user_repository.get_by_id(id).await
    }
}
