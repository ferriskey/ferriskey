use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::domain::client::ports::PostLogoutRedirectUriRepository;
use crate::domain::common::entities::app_errors::CoreError;

#[derive(Debug, Clone)]
pub struct PostgresPostLogoutRedirectUriRepository {
    pub db: DatabaseConnection,
}

impl PostgresPostLogoutRedirectUriRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl PostLogoutRedirectUriRepository for PostgresPostLogoutRedirectUriRepository {
    async fn get_enabled_by_client_id(&self, client_id: Uuid) -> Result<Vec<String>, CoreError> {
        let redirect_uris = crate::entity::post_logout_redirect_uris::Entity::find()
            .filter(crate::entity::post_logout_redirect_uris::Column::ClientId.eq(client_id))
            .filter(crate::entity::post_logout_redirect_uris::Column::Enabled.eq(true))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(redirect_uris.into_iter().map(|uri| uri.value).collect())
    }
}
