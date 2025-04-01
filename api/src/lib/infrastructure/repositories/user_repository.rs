use std::sync::Arc;

use crate::{
    domain::user::{
        entities::{
            error::UserError,
            model::{User, UserConfig},
        },
        ports::UserRepository,
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
    async fn create_user(&self, user_config: UserConfig) -> Result<User, UserError> {
        let user = User::new(user_config);

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
        .map_err(|err| {
            println!("Failed to insert user: {:?}", err);
            UserError::InternalServerError
        });
        Ok(user)
    }
}
