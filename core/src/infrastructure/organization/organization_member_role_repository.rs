use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use tracing::error;
use uuid::Uuid;

use ferriskey_organization::OrganizationMemberRoleRepository;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_timestamp;
use crate::entity::organization_member_roles::{
    ActiveModel as RoleActiveModel, Column as RoleColumn, Entity as RoleEntity,
};

#[derive(Debug, Clone)]
pub struct PostgresOrganizationMemberRoleRepository {
    pub db: DatabaseConnection,
}

impl PostgresOrganizationMemberRoleRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl OrganizationMemberRoleRepository for PostgresOrganizationMemberRoleRepository {
    async fn assign_role(
        &self,
        organization_member_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), CoreError> {
        let (_, timestamp) = generate_timestamp();
        let id = Uuid::new_v7(timestamp);
        let now = Utc::now().fixed_offset();

        RoleEntity::insert(RoleActiveModel {
            id: Set(id),
            organization_member_id: Set(organization_member_id),
            role_id: Set(role_id),
            created_at: Set(now),
        })
        .exec(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to assign organization member role: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(())
    }

    async fn revoke_role(
        &self,
        organization_member_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), CoreError> {
        RoleEntity::delete_many()
            .filter(RoleColumn::OrganizationMemberId.eq(organization_member_id))
            .filter(RoleColumn::RoleId.eq(role_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to revoke organization member role: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn list_role_ids(&self, organization_member_id: Uuid) -> Result<Vec<Uuid>, CoreError> {
        let ids = RoleEntity::find()
            .filter(RoleColumn::OrganizationMemberId.eq(organization_member_id))
            .select_only()
            .column(RoleColumn::RoleId)
            .into_tuple::<Uuid>()
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to list organization member role ids: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(ids)
    }
}
