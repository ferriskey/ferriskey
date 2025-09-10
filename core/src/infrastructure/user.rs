use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::user::entities::User;
use crate::domain::user::ports::UserRepository;
use crate::domain::user::value_objects::{CreateUserRequest, UpdateUserRequest};
use crate::infrastructure::user::repository::PostgresUserRepository;
use uuid::Uuid;

pub mod mappers;
pub mod repositories;
pub mod repository;

#[derive(Clone)]
pub enum UserRepoAny {
    Postgres(PostgresUserRepository),
}

impl UserRepository for UserRepoAny {
    async fn create_user(&self, dto: CreateUserRequest) -> Result<User, CoreError> {
        match self {
            Self::Postgres(repo) => repo.create_user(dto).await,
        }
    }

    async fn get_by_username(&self, username: String, realm_id: Uuid) -> Result<User, CoreError> {
        match self {
            Self::Postgres(repo) => repo.get_by_username(username, realm_id).await,
        }
    }

    async fn get_by_client_id(&self, client_id: Uuid) -> Result<User, CoreError> {
        match self {
            Self::Postgres(repo) => repo.get_by_client_id(client_id).await,
        }
    }

    async fn get_by_id(&self, user_id: Uuid) -> Result<User, CoreError> {
        match self {
            Self::Postgres(repo) => repo.get_by_id(user_id).await,
        }
    }

    async fn find_by_realm_id(&self, realm_id: Uuid) -> Result<Vec<User>, CoreError> {
        match self {
            Self::Postgres(repo) => repo.find_by_realm_id(realm_id).await,
        }
    }

    async fn bulk_delete_user(&self, ids: Vec<Uuid>) -> Result<u64, CoreError> {
        match self {
            Self::Postgres(repo) => repo.bulk_delete_user(ids).await,
        }
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<u64, CoreError> {
        match self {
            Self::Postgres(repo) => repo.delete_user(user_id).await,
        }
    }

    async fn update_user(&self, user_id: Uuid, dto: UpdateUserRequest) -> Result<User, CoreError> {
        match self {
            Self::Postgres(repo) => repo.update_user(user_id, dto).await,
        }
    }
}
