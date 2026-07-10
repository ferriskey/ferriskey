use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::sea_query::Expr;
use sea_orm::sea_query::extension::postgres::PgExpr;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use tracing::error;
use uuid::Uuid;

use ferriskey_organization::{GroupId, GroupMember, GroupMemberDetail, GroupMemberRepository};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_timestamp;
use crate::entity::organization_group_members::{
    ActiveModel as MemberActiveModel, Column as MemberColumn, Entity as MemberEntity,
    Model as MemberModel,
};
use crate::entity::users::{Column as UserColumn, Entity as UserEntity};

#[derive(Debug, Clone)]
pub struct PostgresGroupMemberRepository {
    pub db: DatabaseConnection,
}

impl PostgresGroupMemberRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn member_to_domain(model: MemberModel) -> GroupMember {
    GroupMember {
        id: model.id,
        group_id: GroupId::new(model.group_id),
        user_id: model.user_id,
        created_at: model.created_at.with_timezone(&Utc),
    }
}

/// Case-insensitive `username`/`email` filter, applied to the joined `users` table.
fn search_condition(search: &Option<String>) -> Option<Condition> {
    let term = search
        .as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())?;
    let pattern = format!("%{term}%");
    Some(
        Condition::any()
            .add(Expr::col((UserEntity, UserColumn::Username)).ilike(pattern.clone()))
            .add(Expr::col((UserEntity, UserColumn::Email)).ilike(pattern)),
    )
}

impl GroupMemberRepository for PostgresGroupMemberRepository {
    async fn add_member(&self, group_id: GroupId, user_id: Uuid) -> Result<GroupMember, CoreError> {
        let (_, timestamp) = generate_timestamp();
        let id = Uuid::new_v7(timestamp);
        let now = Utc::now().fixed_offset();

        let model = MemberEntity::insert(MemberActiveModel {
            id: Set(id),
            group_id: Set(group_id.as_uuid()),
            user_id: Set(user_id),
            created_at: Set(now),
        })
        .exec_with_returning(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to add group member: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(member_to_domain(model))
    }

    async fn remove_member(&self, group_id: GroupId, user_id: Uuid) -> Result<(), CoreError> {
        MemberEntity::delete_many()
            .filter(MemberColumn::GroupId.eq(group_id.as_uuid()))
            .filter(MemberColumn::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to remove group member: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn list_members(
        &self,
        group_id: GroupId,
        limit: u32,
        offset: u32,
        search: Option<String>,
    ) -> Result<Vec<GroupMemberDetail>, CoreError> {
        let mut query = MemberEntity::find()
            .filter(MemberColumn::GroupId.eq(group_id.as_uuid()))
            .find_also_related(UserEntity)
            .order_by_asc(UserColumn::Username);

        if let Some(condition) = search_condition(&search) {
            query = query.filter(condition);
        }

        let rows = query
            .limit(limit as u64)
            .offset(offset as u64)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to list group members: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(rows
            .into_iter()
            .filter_map(|(member, user)| {
                user.map(|user| GroupMemberDetail {
                    id: member.id,
                    group_id: GroupId::new(member.group_id),
                    user_id: member.user_id,
                    username: user.username,
                    email: user.email,
                    firstname: user.firstname,
                    lastname: user.lastname,
                    enabled: user.enabled,
                    created_at: member.created_at.with_timezone(&Utc),
                })
            })
            .collect())
    }

    async fn count_members(
        &self,
        group_id: GroupId,
        search: Option<String>,
    ) -> Result<i64, CoreError> {
        let mut query = MemberEntity::find()
            .filter(MemberColumn::GroupId.eq(group_id.as_uuid()))
            .inner_join(UserEntity);

        if let Some(condition) = search_condition(&search) {
            query = query.filter(condition);
        }

        let count = query.count(&self.db).await.map_err(|e| {
            error!("Failed to count group members: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(count as i64)
    }

    async fn get_member(
        &self,
        group_id: GroupId,
        user_id: Uuid,
    ) -> Result<Option<GroupMember>, CoreError> {
        let model = MemberEntity::find()
            .filter(MemberColumn::GroupId.eq(group_id.as_uuid()))
            .filter(MemberColumn::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get group member: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(member_to_domain))
    }
}
