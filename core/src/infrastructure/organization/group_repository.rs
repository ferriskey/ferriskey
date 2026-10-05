use std::collections::HashMap;

use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
    Select,
};
use tracing::error;
use uuid::Uuid;

use ferriskey_domain::realm::scope::Scoped;
use ferriskey_organization::{
    CreateGroupParams, Group, GroupFilter, GroupId, GroupListItem, GroupRepository, GroupSortField,
    Organization, OrganizationId, UpdateGroupParams,
};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_timestamp;
use crate::domain::common::pagination::{Page, PageRequest};
use crate::entity::organization_groups::{
    ActiveModel as GroupActiveModel, Column as GroupColumn, Entity as GroupEntity,
    Model as GroupModel,
};
use crate::infrastructure::pagination::{SortColumn, contains, paginate};

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

fn child_count_select(organization_id: Uuid, parent_ids: Vec<Uuid>) -> Select<GroupEntity> {
    GroupEntity::find()
        .select_only()
        .column(GroupColumn::ParentGroupId)
        .column_as(GroupColumn::Id.count(), "child_count")
        .filter(GroupColumn::OrganizationId.eq(organization_id))
        .filter(GroupColumn::ParentGroupId.is_in(parent_ids))
        .group_by(GroupColumn::ParentGroupId)
}

impl PostgresGroupRepository {
    async fn child_counts(
        &self,
        organization_id: Uuid,
        parent_ids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, u64>, CoreError> {
        if parent_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let rows: Vec<(Option<Uuid>, i64)> = child_count_select(organization_id, parent_ids)
            .into_tuple()
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to count child groups: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(rows
            .into_iter()
            .filter_map(|(parent, count)| Some((parent?, u64::try_from(count).ok()?)))
            .collect())
    }
}

impl SortColumn<GroupEntity> for GroupSortField {
    fn column(&self) -> GroupColumn {
        match self {
            GroupSortField::Name => GroupColumn::Name,
            GroupSortField::CreatedAt => GroupColumn::CreatedAt,
            GroupSortField::UpdatedAt => GroupColumn::UpdatedAt,
        }
    }
}

fn listing_select(organization_id: Uuid, filter: &GroupFilter) -> Select<GroupEntity> {
    GroupEntity::find()
        .filter(GroupColumn::OrganizationId.eq(organization_id))
        .apply_if(filter.name.as_deref(), |select, value| {
            select.filter(contains(GroupColumn::Name, value))
        })
        .apply_if(filter.description.as_deref(), |select, value| {
            select.filter(contains(GroupColumn::Description, value))
        })
        .apply_if(filter.parent_group_id, |select, parent| {
            select.filter(GroupColumn::ParentGroupId.eq(parent))
        })
        .apply_if(filter.is_root, |select, root| {
            select.filter(if root {
                GroupColumn::ParentGroupId.is_null()
            } else {
                GroupColumn::ParentGroupId.is_not_null()
            })
        })
        .apply_if(filter.ids.as_deref(), |select, ids| {
            select.filter(GroupColumn::Id.is_in(ids.iter().copied()))
        })
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

    async fn list(
        &self,
        organization: &Scoped<Organization>,
        request: &PageRequest<GroupFilter, GroupSortField>,
    ) -> Result<Page<GroupListItem>, CoreError> {
        let select = listing_select(organization.get().id.as_uuid(), &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            error!("Failed to list groups: {}", e);
            CoreError::InternalServerError
        })?;
        let counts = self
            .child_counts(
                organization.get().id.as_uuid(),
                models.iter().map(|model| model.id).collect(),
            )
            .await?;

        Ok(Page::new(
            models
                .into_iter()
                .map(|model| {
                    let child_count = counts.get(&model.id).copied().unwrap_or_default();
                    GroupListItem {
                        group: model_to_domain(model),
                        child_count,
                    }
                })
                .collect(),
            total,
            request.page,
            request.limit,
        ))
    }

    async fn update_group(
        &self,
        organization_id: OrganizationId,
        id: GroupId,
        params: UpdateGroupParams,
    ) -> Result<Group, CoreError> {
        let existing = GroupEntity::find_by_id(id.as_uuid())
            .filter(GroupColumn::OrganizationId.eq(organization_id.as_uuid()))
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

    async fn delete_group(
        &self,
        organization_id: OrganizationId,
        id: GroupId,
    ) -> Result<(), CoreError> {
        GroupEntity::delete_many()
            .filter(GroupColumn::Id.eq(id.as_uuid()))
            .filter(GroupColumn::OrganizationId.eq(organization_id.as_uuid()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete group: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod listing_tests {
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;
    use ferriskey_organization::GroupFilter;

    fn sql(filter: &GroupFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_organization() {
        let sql = sql(&GroupFilter::default());
        assert!(
            sql.contains(
                r#""organization_groups"."organization_id" = '00000000-0000-0000-0000-000000000000'"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn text_filters_are_escaped_contains_matches() {
        let sql = sql(&GroupFilter {
            name: Some("a%".to_string()),
            description: Some("b_".to_string()),
            ..GroupFilter::default()
        });
        assert!(
            sql.contains(r#""organization_groups"."name" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""organization_groups"."description" ILIKE E'%b\\_%'"#),
            "{sql}"
        );
    }

    #[test]
    fn parent_group_id_is_an_equality() {
        let parent = Uuid::from_u128(7);
        let sql = sql(&GroupFilter {
            parent_group_id: Some(parent),
            ..GroupFilter::default()
        });
        assert!(
            sql.contains(&format!(
                r#""organization_groups"."parent_group_id" = '{parent}'"#
            )),
            "{sql}"
        );
    }

    #[test]
    fn is_root_tests_the_parent_for_null() {
        let roots = sql(&GroupFilter {
            is_root: Some(true),
            ..GroupFilter::default()
        });
        assert!(
            roots.contains(r#""organization_groups"."parent_group_id" IS NULL"#),
            "{roots}"
        );
        let children = sql(&GroupFilter {
            is_root: Some(false),
            ..GroupFilter::default()
        });
        assert!(
            children.contains(r#""organization_groups"."parent_group_id" IS NOT NULL"#),
            "{children}"
        );
    }

    #[test]
    fn child_counts_are_one_grouped_query_over_the_page() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let sql = super::child_count_select(Uuid::nil(), vec![first, second])
            .build(DbBackend::Postgres)
            .to_string();
        assert!(
            sql.contains(r#"COUNT("organization_groups"."id") AS "child_count""#),
            "{sql}"
        );
        assert!(
            sql.contains(
                r#""organization_groups"."organization_id" = '00000000-0000-0000-0000-000000000000'"#
            ),
            "{sql}"
        );
        assert!(
            sql.contains(&format!(
                r#""organization_groups"."parent_group_id" IN ('{first}', '{second}')"#
            )),
            "{sql}"
        );
        assert!(
            sql.contains(r#"GROUP BY "organization_groups"."parent_group_id""#),
            "{sql}"
        );
    }

    #[test]
    fn ids_filter_is_an_in_list() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let sql = sql(&GroupFilter {
            ids: Some(vec![first, second]),
            ..GroupFilter::default()
        });
        assert!(
            sql.contains(&format!(
                r#""organization_groups"."id" IN ('{first}', '{second}')"#
            )),
            "{sql}"
        );
    }
}
