use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::sea_query::{Expr, SimpleExpr};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, Order, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, QueryTrait, Select, SelectTwo,
};
use tracing::error;
use uuid::Uuid;

use ferriskey_organization::{
    Group, GroupId, GroupMember, GroupMemberDetail, GroupMemberFilter, GroupMemberRepository,
    GroupMemberSortField,
};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_timestamp;
use crate::domain::common::pagination::{Page, PageRequest, SortOrder};
use crate::entity::organization_group_members::{
    ActiveModel as MemberActiveModel, Column as MemberColumn, Entity as MemberEntity,
    Model as MemberModel,
};
use crate::entity::users::{Column as UserColumn, Entity as UserEntity, Model as UserModel};
use crate::infrastructure::pagination::contains;

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

fn listing_select(group_id: Uuid, filter: &GroupMemberFilter) -> Select<MemberEntity> {
    MemberEntity::find()
        .inner_join(UserEntity)
        .filter(MemberColumn::GroupId.eq(group_id))
        .apply_if(filter.username.as_deref(), |select, value| {
            select.filter(contains(UserColumn::Username, value))
        })
        .apply_if(filter.email.as_deref(), |select, value| {
            select.filter(contains(UserColumn::Email, value))
        })
        .apply_if(filter.enabled, |select, value| {
            select.filter(Expr::col((UserEntity, UserColumn::Enabled)).eq(value))
        })
}

fn sort_expr(field: GroupMemberSortField) -> SimpleExpr {
    match field {
        GroupMemberSortField::Username => Expr::col((UserEntity, UserColumn::Username)).into(),
        GroupMemberSortField::Email => Expr::col((UserEntity, UserColumn::Email)).into(),
        GroupMemberSortField::CreatedAt => {
            Expr::col((MemberEntity, MemberColumn::CreatedAt)).into()
        }
    }
}

fn page_select(
    group_id: Uuid,
    request: &PageRequest<GroupMemberFilter, GroupMemberSortField>,
) -> SelectTwo<MemberEntity, UserEntity> {
    let direction = match request.sort.order {
        SortOrder::Asc => Order::Asc,
        SortOrder::Desc => Order::Desc,
    };
    listing_select(group_id, &request.filter)
        .select_also(UserEntity)
        .order_by(sort_expr(request.sort.field), direction.clone())
        .order_by(Expr::col((MemberEntity, MemberColumn::Id)), direction)
        .offset(request.offset())
        .limit(u64::from(request.limit.get()))
}

fn detail_from(member: MemberModel, user: UserModel) -> GroupMemberDetail {
    GroupMemberDetail {
        id: member.id,
        group_id: GroupId::new(member.group_id),
        user_id: member.user_id,
        username: user.username,
        email: user.email,
        firstname: user.firstname,
        lastname: user.lastname,
        enabled: user.enabled,
        created_at: member.created_at.with_timezone(&Utc),
    }
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

    async fn list(
        &self,
        group: &Group,
        request: &PageRequest<GroupMemberFilter, GroupMemberSortField>,
    ) -> Result<Page<GroupMemberDetail>, CoreError> {
        let group_id = group.id.as_uuid();
        let total = listing_select(group_id, &request.filter)
            .count(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to count group members: {}", e);
                CoreError::InternalServerError
            })?;
        let rows = page_select(group_id, request)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to list group members: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(Page::new(
            rows.into_iter()
                .filter_map(|(member, user)| user.map(|user| detail_from(member, user)))
                .collect(),
            total,
            request.page,
            request.limit,
        ))
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

#[cfg(test)]
mod listing_tests {
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::{listing_select, page_select};
    use crate::domain::common::pagination::{PageRequest, Sort, SortOrder};
    use ferriskey_organization::{GroupMemberFilter, GroupMemberSortField};

    fn sql(filter: &GroupMemberFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    fn ordered(field: GroupMemberSortField, order: SortOrder) -> String {
        page_select(
            Uuid::nil(),
            &PageRequest {
                sort: Sort { field, order },
                ..PageRequest::default()
            },
        )
        .build(DbBackend::Postgres)
        .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_group_and_joins_users() {
        let sql = sql(&GroupMemberFilter::default());
        assert!(
            sql.contains(
                r#"INNER JOIN "users" ON "organization_group_members"."user_id" = "users"."id""#
            ),
            "{sql}"
        );
        assert!(
            sql.contains(
                r#""organization_group_members"."group_id" = '00000000-0000-0000-0000-000000000000'"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn text_filters_are_escaped_contains_matches_on_users() {
        let sql = sql(&GroupMemberFilter {
            username: Some("a%".to_string()),
            email: Some("b_".to_string()),
            ..GroupMemberFilter::default()
        });
        assert!(
            sql.contains(r#""users"."username" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
        assert!(sql.contains(r#""users"."email" ILIKE E'%b\\_%'"#), "{sql}");
    }

    #[test]
    fn enabled_is_an_equality_on_users() {
        let sql = sql(&GroupMemberFilter {
            enabled: Some(false),
            ..GroupMemberFilter::default()
        });
        assert!(sql.contains(r#""users"."enabled" = FALSE"#), "{sql}");
    }

    #[test]
    fn every_sort_ends_with_the_membership_id_in_the_same_direction() {
        for (field, column) in [
            (GroupMemberSortField::Username, r#""users"."username""#),
            (GroupMemberSortField::Email, r#""users"."email""#),
            (
                GroupMemberSortField::CreatedAt,
                r#""organization_group_members"."created_at""#,
            ),
        ] {
            for (order, keyword) in [(SortOrder::Asc, "ASC"), (SortOrder::Desc, "DESC")] {
                let sql = ordered(field, order);
                assert!(
                    sql.contains(&format!(
                        r#"ORDER BY {column} {keyword}, "organization_group_members"."id" {keyword}"#
                    )),
                    "{sql}"
                );
            }
        }
    }

    #[test]
    fn the_page_selects_the_user_columns_with_the_membership() {
        let sql = ordered(GroupMemberSortField::default(), SortOrder::default());
        assert!(
            sql.contains(r#""users"."username" AS "B_username""#),
            "{sql}"
        );
        assert!(sql.contains("LIMIT 20 OFFSET 0"), "{sql}");
    }
}
