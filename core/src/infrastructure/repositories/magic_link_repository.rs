use chrono::{DateTime, TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::{entities::app_errors::CoreError, generate_uuid_v7},
        trident::{entities::MagicLink, ports::MagicLinkRepository},
    },
    entity::magic_links::{
        ActiveModel as MagicLinkActiveModel, Column as MagicLinkColumn, Entity as MagicLinkEntity,
        Model as MagicLinkModel,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresMagicLinkRepository {
    pub db: DatabaseConnection,
}

impl PostgresMagicLinkRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<MagicLinkModel> for MagicLink {
    fn from(model: MagicLinkModel) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let expires_at = Utc.from_utc_datetime(&model.expires_at);

        MagicLink {
            id: model.id,
            user_id: model.user_id,
            realm_id: model.realm_id,
            token: model.token,
            created_at,
            expires_at,
        }
    }
}

impl MagicLinkRepository for PostgresMagicLinkRepository {
    async fn create_magic_link(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
        token: String,
        expires_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let active_model = MagicLinkActiveModel {
            id: Set(generate_uuid_v7()),
            user_id: Set(user_id),
            realm_id: Set(realm_id),
            token: Set(token),
            created_at: Set(Utc::now().naive_utc()),
            expires_at: Set(expires_at.naive_utc()),
        };

        active_model.insert(&self.db).await.map_err(|e| {
            error!("Failed to create magic link: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(())
    }

    async fn get_by_token(&self, token: &str) -> Result<Option<MagicLink>, CoreError> {
        let magic_link = MagicLinkEntity::find()
            .filter(MagicLinkColumn::Token.eq(token))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get magic link by token: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(magic_link.map(|ml| ml.into()))
    }

    async fn delete_by_token(&self, token: &str) -> Result<(), CoreError> {
        MagicLinkEntity::delete_many()
            .filter(MagicLinkColumn::Token.eq(token))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete magic link: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn cleanup_expired(&self, realm_id: Uuid) -> Result<(), CoreError> {
        let mut query = MagicLinkEntity::delete_many()
            .filter(MagicLinkColumn::ExpiresAt.lt(Utc::now().naive_utc()));

        query = query.filter(MagicLinkColumn::RealmId.eq(realm_id));

        query.exec(&self.db).await.map_err(|e| {
            error!("Failed to cleanup expired magic links: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(())
    }

    async fn get_user_active_links(
        &self,
        user_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Vec<MagicLink>, CoreError> {
        let magic_links = MagicLinkEntity::find()
            .filter(MagicLinkColumn::UserId.eq(user_id))
            .filter(MagicLinkColumn::RealmId.eq(realm_id))
            .filter(MagicLinkColumn::ExpiresAt.gt(Utc::now().naive_utc()))
            .order_by_desc(MagicLinkColumn::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get user active magic links: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(magic_links.into_iter().map(|ml| ml.into()).collect())
    }
}
