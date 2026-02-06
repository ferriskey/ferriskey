use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use tracing::instrument;
use uuid::Uuid;

use crate::domain::abyss::identity_provider::IdentityProviderId;
use crate::domain::abyss::identity_provider::broker::{
    CreateIdentityProviderLinkRequest, IdentityProviderLink, IdentityProviderLinkRepository,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_uuid_v7;
use crate::entity::identity_provider_links::{
    ActiveModel, Column, Entity as IdentityProviderLinkEntity,
};

/// PostgreSQL implementation of the IdentityProviderLinkRepository trait
#[derive(Debug, Clone)]
pub struct PostgresIdentityProviderLinkRepository {
    db: DatabaseConnection,
}

impl PostgresIdentityProviderLinkRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl IdentityProviderLinkRepository for PostgresIdentityProviderLinkRepository {
    #[instrument(skip(self, request), fields(user_id = %request.user_id, idp_id = %request.identity_provider_id))]
    async fn create(
        &self,
        request: CreateIdentityProviderLinkRequest,
    ) -> Result<IdentityProviderLink, CoreError> {
        let now = Utc::now().fixed_offset();

        let payload = ActiveModel {
            id: Set(generate_uuid_v7()),
            user_id: Set(request.user_id),
            identity_provider_id: Set(request.identity_provider_id),
            identity_provider_user_id: Set(request.identity_provider_user_id),
            identity_provider_username: Set(request.identity_provider_username),
            token: Set(request.token),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let link = payload.insert(&self.db).await.map_err(|e| {
            tracing::error!("Failed to create identity provider link: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(link.into())
    }

    #[instrument(skip(self), fields(idp_id = ?identity_provider_id, external_id = %external_user_id))]
    async fn get_by_provider_and_external_id(
        &self,
        identity_provider_id: IdentityProviderId,
        external_user_id: &str,
    ) -> Result<Option<IdentityProviderLink>, CoreError> {
        let link = IdentityProviderLinkEntity::find()
            .filter(Column::IdentityProviderId.eq::<Uuid>(identity_provider_id.into()))
            .filter(Column::IdentityProviderUserId.eq(external_user_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get identity provider link: {}", e);
                CoreError::InternalServerError
            })?
            .map(IdentityProviderLink::from);

        Ok(link)
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn get_by_user_id(&self, user_id: Uuid) -> Result<Vec<IdentityProviderLink>, CoreError> {
        let links = IdentityProviderLinkEntity::find()
            .filter(Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get identity provider links by user: {}", e);
                CoreError::InternalServerError
            })?
            .into_iter()
            .map(IdentityProviderLink::from)
            .collect();

        Ok(links)
    }

    #[instrument(skip(self), fields(user_id = %user_id, idp_id = ?identity_provider_id))]
    async fn get_by_user_and_provider(
        &self,
        user_id: Uuid,
        identity_provider_id: IdentityProviderId,
    ) -> Result<Option<IdentityProviderLink>, CoreError> {
        let link = IdentityProviderLinkEntity::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::IdentityProviderId.eq::<Uuid>(identity_provider_id.into()))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to get identity provider link by user and provider: {}",
                    e
                );
                CoreError::InternalServerError
            })?
            .map(IdentityProviderLink::from);

        Ok(link)
    }

    #[instrument(skip(self), fields(link_id = %id))]
    async fn update_token(&self, id: Uuid, token: Option<String>) -> Result<(), CoreError> {
        let existing = IdentityProviderLinkEntity::find()
            .filter(Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find identity provider link for update: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::LinkNotFound)?;

        let mut link: ActiveModel = existing.into();
        link.token = Set(token);
        link.updated_at = Set(Utc::now().fixed_offset());

        link.update(&self.db).await.map_err(|e| {
            tracing::error!("Failed to update identity provider link token: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(())
    }

    #[instrument(skip(self), fields(link_id = %id))]
    async fn delete(&self, id: Uuid) -> Result<(), CoreError> {
        IdentityProviderLinkEntity::delete_many()
            .filter(Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete identity provider link: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn delete_by_user_id(&self, user_id: Uuid) -> Result<u64, CoreError> {
        let result = IdentityProviderLinkEntity::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete identity provider links by user: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected)
    }
}

/// Convert from SeaORM model to domain entity
impl From<crate::entity::identity_provider_links::Model> for IdentityProviderLink {
    fn from(model: crate::entity::identity_provider_links::Model) -> Self {
        Self {
            id: model.id,
            user_id: model.user_id,
            identity_provider_id: model.identity_provider_id.into(),
            identity_provider_user_id: model.identity_provider_user_id,
            identity_provider_username: model.identity_provider_username,
            token: model.token,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
        }
    }
}
