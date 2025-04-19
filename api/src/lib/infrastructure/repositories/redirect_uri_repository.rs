use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::client::{
    entities::{redirect_uri::RedirectUri, redirect_uri_error::RedirectUriError},
    ports::redirect_uri_repository::RedirectUriRepository,
};

#[derive(Debug, Clone)]
pub struct PostgresRedirectUriRepository {
    pub pool: PgPool,
}
impl PostgresRedirectUriRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RedirectUriRepository for PostgresRedirectUriRepository {
    async fn create_redirect_uri(
        &self,
        client_id: Uuid,
        value: String,
        enabled: bool,
    ) -> Result<RedirectUri, RedirectUriError> {
        let redirect_uri = RedirectUri::new(client_id, value, enabled);

        sqlx::query!(
            r#"
            INSERT INTO redirect_uris (id, client_id, value, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            redirect_uri.id,
            redirect_uri.client_id,
            redirect_uri.value,
            redirect_uri.created_at,
            redirect_uri.updated_at
        )
        .execute(&self.pool)
        .await
        .map_err(|_| RedirectUriError::DatabaseError)?;

        Ok(redirect_uri)
    }

    async fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<RedirectUri>, RedirectUriError> {
        todo!()
    }

    async fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<RedirectUri>, RedirectUriError> {
        todo!()
    }

    async fn update_enabled(
        &self,
        id: Uuid,
        enabled: bool,
    ) -> Result<RedirectUri, RedirectUriError> {
        todo!()
    }

    async fn delete(&self, id: Uuid) -> Result<(), RedirectUriError> {
        todo!()
    }
}
