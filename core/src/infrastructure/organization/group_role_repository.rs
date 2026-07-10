use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use tracing::error;
use uuid::Uuid;

use ferriskey_organization::{GroupId, GroupRoleMapping, GroupRoleRepository};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_timestamp;
use crate::entity::organization_group_roles::{
    ActiveModel as RoleActiveModel, Column as RoleColumn, Entity as RoleEntity, Model as RoleModel,
};

#[derive(Debug, Clone)]
pub struct PostgresGroupRoleRepository {
    pub db: DatabaseConnection,
}

impl PostgresGroupRoleRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn model_to_domain(model: RoleModel) -> GroupRoleMapping {
    GroupRoleMapping {
        id: model.id,
        group_id: GroupId::new(model.group_id),
        role_id: model.role_id,
        created_at: model.created_at.with_timezone(&Utc),
    }
}

impl GroupRoleRepository for PostgresGroupRoleRepository {
    async fn assign_role(
        &self,
        group_id: GroupId,
        role_id: Uuid,
    ) -> Result<GroupRoleMapping, CoreError> {
        let (_, timestamp) = generate_timestamp();
        let id = Uuid::new_v7(timestamp);
        let now = Utc::now().fixed_offset();

        let model = RoleEntity::insert(RoleActiveModel {
            id: Set(id),
            group_id: Set(group_id.as_uuid()),
            role_id: Set(role_id),
            created_at: Set(now),
        })
        .exec_with_returning(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to assign group role: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model_to_domain(model))
    }

    async fn revoke_role(&self, group_id: GroupId, role_id: Uuid) -> Result<(), CoreError> {
        RoleEntity::delete_many()
            .filter(RoleColumn::GroupId.eq(group_id.as_uuid()))
            .filter(RoleColumn::RoleId.eq(role_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to revoke group role: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn list_role_ids(&self, group_id: GroupId) -> Result<Vec<Uuid>, CoreError> {
        let ids = RoleEntity::find()
            .filter(RoleColumn::GroupId.eq(group_id.as_uuid()))
            .select_only()
            .column(RoleColumn::RoleId)
            .into_tuple::<Uuid>()
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to list group role ids: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(ids)
    }
}
