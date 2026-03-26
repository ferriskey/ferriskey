use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::error;
use uuid::Uuid;

use ferriskey_organization::{OrganizationId, OrganizationMember, OrganizationMemberRepository};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_timestamp;
use crate::entity::organization_members::{
    ActiveModel as MemberActiveModel, Column as MemberColumn, Entity as MemberEntity,
    Model as MemberModel,
};

#[derive(Debug, Clone)]
pub struct PostgresOrganizationMemberRepository {
    pub db: DatabaseConnection,
}

impl PostgresOrganizationMemberRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn model_to_domain(model: MemberModel) -> OrganizationMember {
    OrganizationMember {
        id: model.id,
        organization_id: OrganizationId::new(model.organization_id),
        user_id: model.user_id,
        created_at: model.created_at.with_timezone(&Utc),
    }
}

impl OrganizationMemberRepository for PostgresOrganizationMemberRepository {
    async fn add_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> Result<OrganizationMember, CoreError> {
        let (_, timestamp) = generate_timestamp();
        let id = Uuid::new_v7(timestamp);
        let now = Utc::now().fixed_offset();

        let model = MemberEntity::insert(MemberActiveModel {
            id: Set(id),
            organization_id: Set(organization_id.as_uuid()),
            user_id: Set(user_id),
            created_at: Set(now),
        })
        .exec_with_returning(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to add organization member: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model_to_domain(model))
    }

    async fn remove_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> Result<(), CoreError> {
        MemberEntity::delete_many()
            .filter(MemberColumn::OrganizationId.eq(organization_id.as_uuid()))
            .filter(MemberColumn::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to remove organization member: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn list_members(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<OrganizationMember>, CoreError> {
        let models = MemberEntity::find()
            .filter(MemberColumn::OrganizationId.eq(organization_id.as_uuid()))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to list organization members: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(model_to_domain).collect())
    }

    async fn list_organizations_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<OrganizationMember>, CoreError> {
        let models = MemberEntity::find()
            .filter(MemberColumn::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to list organizations for user: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(model_to_domain).collect())
    }

    async fn get_member(
        &self,
        organization_id: OrganizationId,
        user_id: Uuid,
    ) -> Result<Option<OrganizationMember>, CoreError> {
        let model = MemberEntity::find()
            .filter(MemberColumn::OrganizationId.eq(organization_id.as_uuid()))
            .filter(MemberColumn::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get organization member: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(model_to_domain))
    }
}
