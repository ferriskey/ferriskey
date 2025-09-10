use crate::{
    domain::common::entities::app_errors::CoreError,
    entity::redirect_uris::{ActiveModel, Entity as RedirectUriEntity},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::client::{entities::redirect_uri::RedirectUri, ports::RedirectUriRepository};

#[derive(Debug, Clone)]
pub struct PostgresRedirectUriRepository {
    pub db: DatabaseConnection,
}
impl PostgresRedirectUriRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl RedirectUriRepository for PostgresRedirectUriRepository {
    async fn create_redirect_uri(
        &self,
        client_id: Uuid,
        value: String,
        enabled: bool,
    ) -> Result<RedirectUri, CoreError> {
        let redirect_uri = RedirectUri::new(client_id, value, enabled);

        let payload = ActiveModel {
            id: Set(redirect_uri.id),
            client_id: Set(redirect_uri.client_id),
            value: Set(redirect_uri.value),
            enabled: Set(redirect_uri.enabled),
            created_at: Set(redirect_uri.created_at.naive_utc()),
            updated_at: Set(redirect_uri.updated_at.naive_utc()),
        };

        let t = payload
            .insert(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(t.into())
    }

    async fn get_by_client_id(&self, client_id: Uuid) -> Result<Vec<RedirectUri>, CoreError> {
        let redirect_uris = RedirectUriEntity::find()
            .filter(crate::entity::redirect_uris::Column::ClientId.eq(client_id))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::RedirectUriNotFound)?;

        let redirect_uris = redirect_uris
            .into_iter()
            .map(RedirectUri::from)
            .collect::<Vec<RedirectUri>>();

        Ok(redirect_uris)
    }

    async fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<RedirectUri>, CoreError> {
        let redirect_uris = RedirectUriEntity::find()
            .filter(crate::entity::redirect_uris::Column::ClientId.eq(client_id))
            .filter(crate::entity::redirect_uris::Column::Enabled.eq(true))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::RedirectUriNotFound)?
            .into_iter()
            .map(RedirectUri::from)
            .collect::<Vec<RedirectUri>>();

        Ok(redirect_uris)
    }

    async fn update_enabled(&self, id: Uuid, enabled: bool) -> Result<RedirectUri, CoreError> {
        let redirect_uri = RedirectUriEntity::find()
            .filter(crate::entity::redirect_uris::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::RedirectUriNotFound)?;

        if let Some(redirect_uri) = redirect_uri {
            let mut redirect_uri: ActiveModel = redirect_uri.into();
            redirect_uri.enabled = Set(enabled);

            let redirect_uri = redirect_uri
                .update(&self.db)
                .await
                .map_err(|_| CoreError::InternalServerError)?;

            let redirect_uri = redirect_uri.into();

            Ok(redirect_uri)
        } else {
            Err(CoreError::RedirectUriNotFound)
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), CoreError> {
        RedirectUriEntity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(())
    }
}
