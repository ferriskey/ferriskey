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
        sqlx::query_as!(
            RedirectUri,
            r#"
            SELECT * FROM redirect_uris WHERE client_id = $1
            "#,
            client_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| RedirectUriError::DatabaseError)
    }

    async fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<RedirectUri>, RedirectUriError> {
        sqlx::query_as!(
            RedirectUri,
            r#"
            SELECT * FROM redirect_uris WHERE client_id = $1
            AND enabled = true
            "#,
            client_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| RedirectUriError::DatabaseError)
    }

    async fn update_enabled(
        &self,
        id: Uuid,
        enabled: bool,
    ) -> Result<RedirectUri, RedirectUriError> {
        sqlx::query_as!(
            RedirectUri,
            r#"
            UPDATE redirect_uris SET enabled = $1, updated_at = NOW() WHERE id = $2
            RETURNING id, client_id, value, enabled, created_at, updated_at
            "#,
            enabled,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| RedirectUriError::DatabaseError)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RedirectUriError> {
        let res = sqlx::query!(
            r#"
            DELETE FROM redirect_uris WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|_| RedirectUriError::DatabaseError)?;

        if res.rows_affected() == 0 {
            return Err(RedirectUriError::NotFound);
        }

        Ok(())
    }
}
