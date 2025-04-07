use std::sync::Arc;

use uuid::Uuid;

use crate::{
    domain::user::{
        dtos::user_dto::CreateUserDto,
        entities::{error::UserError, model::User},
        ports::user_repository::UserRepository,
    },
    infrastructure::db::postgres::Postgres,
};

#[derive(Debug, Clone)]
pub struct PostgresUserRepository {
    pub postgres: Arc<Postgres>,
}

impl PostgresUserRepository {
    pub fn new(postgres: Arc<Postgres>) -> Self {
        Self { postgres }
    }
}

impl UserRepository for PostgresUserRepository {
    async fn create_user(&self, dto: CreateUserDto) -> Result<User, UserError> {
        let user = User::from_dto(dto);

        let _ = sqlx::query_as!(User, r#"
        INSERT INTO users (id, realm_id, username, firstname, lastname, email, email_verified, enabled, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        user.id,
        user.realm_id,
        user.username,
        user.firstname,
        user.lastname,
        user.email,
        user.email_verified,
        user.enabled,
        user.created_at,
        user.updated_at
        )
        .execute(&*self.postgres.get_pool())
        .await
        .map_err(|_| UserError::InternalServerError)?;
        Ok(user)
    }

    async fn get_by_username(&self, username: String, realm_id: Uuid) -> Result<User, UserError> {
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE username = $1 AND realm_id = $2",
            username,
            realm_id
        )
        .fetch_one(&*self.postgres.get_pool())
        .await
        .map_err(|_| UserError::NotFound)?;
        Ok(user)
    }
}
