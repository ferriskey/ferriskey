use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use tracing::error;

use ferriskey_organization::{
    CreateGroupParams, Group, GroupId, GroupRepository, OrganizationId, UpdateGroupParams,
};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_timestamp;
use crate::entity::organization_groups::{
    ActiveModel as GroupActiveModel, Column as GroupColumn, Entity as GroupEntity,
    Model as GroupModel,
};

#[derive(Debug, Clone)]
pub struct PostgresGroupRepository {
    pub db: DatabaseConnection,
}

impl PostgresGroupRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn model_to_domain(model: GroupModel) -> Group {
    Group {
        id: GroupId::new(model.id),
        organization_id: OrganizationId::new(model.organization_id),
        parent_group_id: model.parent_group_id.map(GroupId::new),
        name: model.name,
        description: model.description,
        created_at: model.created_at.with_timezone(&Utc),
        updated_at: model.updated_at.with_timezone(&Utc),
    }
}

impl GroupRepository for PostgresGroupRepository {
    async fn create_group(&self, params: CreateGroupParams) -> Result<Group, CoreError> {
        let (_, timestamp) = generate_timestamp();
        let id = uuid::Uuid::new_v7(timestamp);
        let now = Utc::now().fixed_offset();

        let model = GroupEntity::insert(GroupActiveModel {
            id: Set(id),
            organization_id: Set(params.organization_id.as_uuid()),
            parent_group_id: Set(params.parent_group_id.map(|p| p.as_uuid())),
            name: Set(params.name),
            description: Set(params.description),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .exec_with_returning(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to create group: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model_to_domain(model))
    }

    async fn get_group_by_id(&self, id: GroupId) -> Result<Option<Group>, CoreError> {
        let model = GroupEntity::find_by_id(id.as_uuid())
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get group: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(model_to_domain))
    }

    async fn list_groups_by_organization(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Group>, CoreError> {
        let models = GroupEntity::find()
            .filter(GroupColumn::OrganizationId.eq(organization_id.as_uuid()))
            .order_by_asc(GroupColumn::Name)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to list groups: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(model_to_domain).collect())
    }

    async fn update_group(
        &self,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> Result<Group, CoreError> {
        let existing = GroupEntity::find_by_id(id.as_uuid())
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to load group for update: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active: GroupActiveModel = existing.into();
        if let Some(name) = params.name {
            active.name = Set(name);
        }
        if let Some(description) = params.description {
            active.description = Set(Some(description));
        }
        if let Some(parent) = params.parent_group_id {
            active.parent_group_id = Set(parent.map(|p| p.as_uuid()));
        }
        active.updated_at = Set(Utc::now().fixed_offset());

        let model = GroupEntity::update(active)
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to update group: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model_to_domain(model))
    }

    async fn delete_group(&self, id: GroupId) -> Result<(), CoreError> {
        GroupEntity::delete_by_id(id.as_uuid())
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete group: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }
}
