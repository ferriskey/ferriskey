use chrono::{TimeZone, Utc};
use entity::redirect_uris::{ActiveModel, Entity as RedirectUriEntity};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use sqlx::{Executor, PgPool};
use uuid::Uuid;

use crate::domain::client::{
    entities::{redirect_uri::RedirectUri, redirect_uri_error::RedirectUriError},
    ports::redirect_uri_repository::RedirectUriRepository,
};

impl From<entity::redirect_uris::Model> for RedirectUri {
    fn from(model: entity::redirect_uris::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let updated_at = Utc.from_utc_datetime(&model.updated_at);

        RedirectUri {
            id: model.id,
            client_id: model.client_id,
            value: model.value,
            enabled: model.enabled,
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresRedirectUriRepository {
    pub pool: PgPool,
    pub db: DatabaseConnection,
}
impl PostgresRedirectUriRepository {
    pub fn new(pool: PgPool, db: DatabaseConnection) -> Self {
        Self { pool, db }
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
            .map_err(|_| RedirectUriError::DatabaseError)?;

        Ok(t.into())
    }

    async fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<RedirectUri>, RedirectUriError> {
        let redirect_uris = RedirectUriEntity::find()
            .filter(entity::redirect_uris::Column::ClientId.eq(client_id))
            .all(&self.db)
            .await
            .map_err(|_| RedirectUriError::DatabaseError)?;

        let redirect_uris = redirect_uris
            .into_iter()
            .map(RedirectUri::from)
            .collect::<Vec<RedirectUri>>();

        Ok(redirect_uris)
    }

    async fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<RedirectUri>, RedirectUriError> {
        let redirect_uris = RedirectUriEntity::find()
            .filter(entity::redirect_uris::Column::ClientId.eq(client_id))
            .filter(entity::redirect_uris::Column::Enabled.eq(true))
            .all(&self.db)
            .await
            .map_err(|_| RedirectUriError::DatabaseError)?
            .into_iter()
            .map(RedirectUri::from)
            .collect::<Vec<RedirectUri>>();

        Ok(redirect_uris)
    }

    async fn update_enabled(
        &self,
        id: Uuid,
        enabled: bool,
    ) -> Result<RedirectUri, RedirectUriError> {
        let redirect_uri = RedirectUriEntity::find()
            .filter(entity::redirect_uris::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|_| RedirectUriError::DatabaseError)?;

        if let Some(redirect_uri) = redirect_uri {
            let mut redirect_uri: ActiveModel = redirect_uri.into();
            redirect_uri.enabled = Set(enabled);

            let redirect_uri = redirect_uri
                .update(&self.db)
                .await
                .map_err(|_| RedirectUriError::DatabaseError)?;

            let redirect_uri = redirect_uri.into();

            Ok(redirect_uri)
        } else {
            Err(RedirectUriError::NotFound)
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), RedirectUriError> {
        RedirectUriEntity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|_| RedirectUriError::DatabaseError)?;

        Ok(())
    }
}
