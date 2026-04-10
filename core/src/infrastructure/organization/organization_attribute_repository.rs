use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::error;
use uuid::Uuid;

use ferriskey_organization::{
    OrganizationAttribute, OrganizationAttributeRepository, OrganizationId,
};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_timestamp;
use crate::entity::organization_attributes::{
    ActiveModel as AttributeActiveModel, Column as AttributeColumn, Entity as AttributeEntity,
    Model as AttributeModel,
};

#[derive(Debug, Clone)]
pub struct PostgresOrganizationAttributeRepository {
    pub db: DatabaseConnection,
}

impl PostgresOrganizationAttributeRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn model_to_domain(model: AttributeModel) -> OrganizationAttribute {
    OrganizationAttribute {
        id: model.id,
        organization_id: OrganizationId::new(model.organization_id),
        key: model.key,
        value: model.value,
        created_at: model.created_at.with_timezone(&Utc),
    }
}

impl OrganizationAttributeRepository for PostgresOrganizationAttributeRepository {
    async fn list_attributes(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<OrganizationAttribute>, CoreError> {
        let models = AttributeEntity::find()
            .filter(AttributeColumn::OrganizationId.eq(organization_id.as_uuid()))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to list organization attributes: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(model_to_domain).collect())
    }

    async fn upsert_attribute(
        &self,
        organization_id: OrganizationId,
        key: String,
        value: String,
    ) -> Result<OrganizationAttribute, CoreError> {
        let existing = AttributeEntity::find()
            .filter(AttributeColumn::OrganizationId.eq(organization_id.as_uuid()))
            .filter(AttributeColumn::Key.eq(key.as_str()))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to find organization attribute: {}", e);
                CoreError::InternalServerError
            })?;

        let model = if let Some(existing) = existing {
            let active_model = AttributeActiveModel {
                id: Set(existing.id),
                organization_id: Set(existing.organization_id),
                key: Set(existing.key),
                value: Set(value),
                created_at: Set(existing.created_at),
            };

            AttributeEntity::update(active_model)
                .filter(AttributeColumn::Id.eq(existing.id))
                .exec(&self.db)
                .await
                .map_err(|e| {
                    error!("Failed to update organization attribute: {}", e);
                    CoreError::InternalServerError
                })?
        } else {
            let (_, timestamp) = generate_timestamp();
            let id = Uuid::new_v7(timestamp);
            let now = Utc::now().fixed_offset();

            AttributeEntity::insert(AttributeActiveModel {
                id: Set(id),
                organization_id: Set(organization_id.as_uuid()),
                key: Set(key),
                value: Set(value),
                created_at: Set(now),
            })
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to insert organization attribute: {}", e);
                CoreError::InternalServerError
            })?
        };

        Ok(model_to_domain(model))
    }

    async fn delete_attribute(
        &self,
        organization_id: OrganizationId,
        key: &str,
    ) -> Result<(), CoreError> {
        AttributeEntity::delete_many()
            .filter(AttributeColumn::OrganizationId.eq(organization_id.as_uuid()))
            .filter(AttributeColumn::Key.eq(key))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete organization attribute: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }
}
