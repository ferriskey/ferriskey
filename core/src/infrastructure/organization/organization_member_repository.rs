use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::sea_query::{Expr, SimpleExpr};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryTrait, Select,
};
use tracing::error;
use uuid::Uuid;

use ferriskey_domain::realm::scope::Scoped;
use ferriskey_organization::{
    Organization, OrganizationId, OrganizationMember, OrganizationMemberFilter,
    OrganizationMemberRepository, OrganizationMemberSortField,
};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_timestamp;
use crate::domain::common::pagination::{Page, PageRequest};
use crate::domain::realm::entities::RealmId;
use crate::entity::organization_members::{
    ActiveModel as MemberActiveModel, Column as MemberColumn, Entity as MemberEntity,
    Model as MemberModel,
};
use crate::entity::users::{Column as UserColumn, Entity as UserEntity};
use crate::infrastructure::pagination::{SortExpr, contains, page_by_expr};

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

fn listing_select(
    organization_id: Uuid,
    filter: &OrganizationMemberFilter,
) -> Select<MemberEntity> {
    MemberEntity::find()
        .inner_join(UserEntity)
        .filter(MemberColumn::OrganizationId.eq(organization_id))
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

impl SortExpr for OrganizationMemberSortField {
    fn expr(&self) -> SimpleExpr {
        match self {
            OrganizationMemberSortField::Username => {
                Expr::col((UserEntity, UserColumn::Username)).into()
            }
            OrganizationMemberSortField::Email => Expr::col((UserEntity, UserColumn::Email)).into(),
            OrganizationMemberSortField::CreatedAt => {
                Expr::col((MemberEntity, MemberColumn::CreatedAt)).into()
            }
        }
    }
}

fn tie_breaker() -> SimpleExpr {
    Expr::col((MemberEntity, MemberColumn::Id)).into()
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

    async fn list(
        &self,
        organization: &Scoped<Organization>,
        request: &PageRequest<OrganizationMemberFilter, OrganizationMemberSortField>,
    ) -> Result<Page<OrganizationMember>, CoreError> {
        let select = listing_select(organization.get().id.as_uuid(), &request.filter);
        let total = select.clone().count(&self.db).await.map_err(|e| {
            error!("Failed to count organization members: {}", e);
            CoreError::InternalServerError
        })?;
        let models = page_by_expr(select, request, tie_breaker())
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to list organization members: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(Page::new(
            models.into_iter().map(model_to_domain).collect(),
            total,
            request.page,
            request.limit,
        ))
    }

    async fn list_organizations_for_user(
        &self,
        realm_id: RealmId,
        user_id: Uuid,
    ) -> Result<Vec<OrganizationMember>, CoreError> {
        let models = MemberEntity::find()
            .filter(MemberColumn::UserId.eq(user_id))
            .inner_join(crate::entity::organizations::Entity)
            .filter(crate::entity::organizations::Column::RealmId.eq(Uuid::from(realm_id)))
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

#[cfg(test)]
mod listing_tests {
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::{listing_select, tie_breaker};
    use crate::domain::common::pagination::{PageRequest, Sort, SortOrder};
    use crate::infrastructure::pagination::page_by_expr;
    use ferriskey_organization::{OrganizationMemberFilter, OrganizationMemberSortField};

    fn sql(filter: &OrganizationMemberFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    fn ordered(field: OrganizationMemberSortField, order: SortOrder) -> String {
        page_by_expr(
            listing_select(Uuid::nil(), &OrganizationMemberFilter::default()),
            &PageRequest::<OrganizationMemberFilter, OrganizationMemberSortField> {
                sort: Sort { field, order },
                ..PageRequest::default()
            },
            tie_breaker(),
        )
        .build(DbBackend::Postgres)
        .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_organization_and_joins_users() {
        let sql = sql(&OrganizationMemberFilter::default());
        assert!(
            sql.contains(
                r#"INNER JOIN "users" ON "organization_members"."user_id" = "users"."id""#
            ),
            "{sql}"
        );
        assert!(
            sql.contains(
                r#""organization_members"."organization_id" = '00000000-0000-0000-0000-000000000000'"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn text_filters_are_escaped_contains_matches_on_users() {
        let sql = sql(&OrganizationMemberFilter {
            username: Some("a%".to_string()),
            email: Some("b_".to_string()),
            ..OrganizationMemberFilter::default()
        });
        assert!(
            sql.contains(r#""users"."username" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
        assert!(sql.contains(r#""users"."email" ILIKE E'%b\\_%'"#), "{sql}");
    }

    #[test]
    fn enabled_is_an_equality_on_users() {
        let sql = sql(&OrganizationMemberFilter {
            enabled: Some(false),
            ..OrganizationMemberFilter::default()
        });
        assert!(sql.contains(r#""users"."enabled" = FALSE"#), "{sql}");
    }

    #[test]
    fn every_sort_ends_with_the_membership_id_in_the_same_direction() {
        for (field, column) in [
            (
                OrganizationMemberSortField::Username,
                r#""users"."username""#,
            ),
            (OrganizationMemberSortField::Email, r#""users"."email""#),
            (
                OrganizationMemberSortField::CreatedAt,
                r#""organization_members"."created_at""#,
            ),
        ] {
            for (order, keyword) in [(SortOrder::Asc, "ASC"), (SortOrder::Desc, "DESC")] {
                let sql = ordered(field, order);
                assert!(
                    sql.contains(&format!(
                        r#"ORDER BY {column} {keyword}, "organization_members"."id" {keyword}"#
                    )),
                    "{sql}"
                );
            }
        }
    }

    #[test]
    fn the_page_selects_only_the_membership_columns() {
        let sql = ordered(OrganizationMemberSortField::default(), SortOrder::default());
        assert!(
            sql.starts_with(r#"SELECT "organization_members"."id", "organization_members"."organization_id", "organization_members"."user_id", "organization_members"."created_at" FROM"#),
            "{sql}"
        );
        assert!(sql.contains("LIMIT 20 OFFSET 0"), "{sql}");
    }
}
