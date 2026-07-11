use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::error;
use uuid::Uuid;

use ferriskey_organization::{GroupAttribute, GroupAttributeRepository, GroupId};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_timestamp;
use crate::entity::organization_group_attributes::{
    ActiveModel as AttributeActiveModel, Column as AttributeColumn, Entity as AttributeEntity,
    Model as AttributeModel,
};

#[derive(Debug, Clone)]
pub struct PostgresGroupAttributeRepository {
    pub db: DatabaseConnection,
}

impl PostgresGroupAttributeRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn model_to_domain(model: AttributeModel) -> GroupAttribute {
    GroupAttribute {
        id: model.id,
        group_id: GroupId::new(model.group_id),
        key: model.key,
        value: model.value,
        created_at: model.created_at.with_timezone(&Utc),
    }
}

impl GroupAttributeRepository for PostgresGroupAttributeRepository {
    async fn list_attributes(&self, group_id: GroupId) -> Result<Vec<GroupAttribute>, CoreError> {
        let models = AttributeEntity::find()
            .filter(AttributeColumn::GroupId.eq(group_id.as_uuid()))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to list group attributes: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(model_to_domain).collect())
    }

    async fn upsert_attribute(
        &self,
        group_id: GroupId,
        key: String,
        value: String,
    ) -> Result<GroupAttribute, CoreError> {
        let existing = AttributeEntity::find()
            .filter(AttributeColumn::GroupId.eq(group_id.as_uuid()))
            .filter(AttributeColumn::Key.eq(key.as_str()))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to find group attribute: {}", e);
                CoreError::InternalServerError
            })?;

        let model = if let Some(existing) = existing {
            let active_model = AttributeActiveModel {
                id: Set(existing.id),
                group_id: Set(existing.group_id),
                key: Set(existing.key),
                value: Set(value),
                created_at: Set(existing.created_at),
            };

            AttributeEntity::update(active_model)
                .filter(AttributeColumn::Id.eq(existing.id))
                .exec(&self.db)
                .await
                .map_err(|e| {
                    error!("Failed to update group attribute: {}", e);
                    CoreError::InternalServerError
                })?
        } else {
            let (_, timestamp) = generate_timestamp();
            let id = Uuid::new_v7(timestamp);
            let now = Utc::now().fixed_offset();

            AttributeEntity::insert(AttributeActiveModel {
                id: Set(id),
                group_id: Set(group_id.as_uuid()),
                key: Set(key),
                value: Set(value),
                created_at: Set(now),
            })
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to insert group attribute: {}", e);
                CoreError::InternalServerError
            })?
        };

        Ok(model_to_domain(model))
    }

    async fn delete_attribute(&self, group_id: GroupId, key: &str) -> Result<(), CoreError> {
        AttributeEntity::delete_many()
            .filter(AttributeColumn::GroupId.eq(group_id.as_uuid()))
            .filter(AttributeColumn::Key.eq(key))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete group attribute: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }
}
