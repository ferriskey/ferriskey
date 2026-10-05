use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QuerySelect, QueryTrait, Select,
};
use tracing::error;
use uuid::Uuid;

use ferriskey_domain::realm::RealmId;
use ferriskey_domain::realm::scope::{RealmScope, Scoped, Unscoped};
use ferriskey_organization::{
    CreateOrganizationParams, Organization, OrganizationFilter, OrganizationId,
    OrganizationRepository, OrganizationSortField, UpdateOrganizationParams,
};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_timestamp;
use crate::domain::common::pagination::{Page, PageRequest};
use crate::entity::organization_members::{
    Column as OrganizationMemberColumn, Entity as OrganizationMemberEntity,
};
use crate::entity::organizations::{
    ActiveModel as OrganizationActiveModel, Column as OrganizationColumn,
    Entity as OrganizationEntity, Model as OrganizationModel,
};
use crate::infrastructure::pagination::{SortColumn, contains, paginate, within};

#[derive(Debug, Clone)]
pub struct PostgresOrganizationRepository {
    pub db: DatabaseConnection,
}

impl PostgresOrganizationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn model_to_domain(model: OrganizationModel) -> Organization {
    Organization {
        id: OrganizationId::new(model.id),
        realm_id: RealmId::new(model.realm_id),
        name: model.name,
        alias: model.alias,
        domain: model.domain,
        redirect_url: model.redirect_url,
        description: model.description,
        enabled: model.enabled,
        created_at: model.created_at.with_timezone(&Utc),
        updated_at: model.updated_at.with_timezone(&Utc),
    }
}

impl SortColumn<OrganizationEntity> for OrganizationSortField {
    fn column(&self) -> OrganizationColumn {
        match self {
            OrganizationSortField::Name => OrganizationColumn::Name,
            OrganizationSortField::Alias => OrganizationColumn::Alias,
            OrganizationSortField::Enabled => OrganizationColumn::Enabled,
            OrganizationSortField::CreatedAt => OrganizationColumn::CreatedAt,
            OrganizationSortField::UpdatedAt => OrganizationColumn::UpdatedAt,
        }
    }
}

fn search(value: &str) -> Condition {
    Condition::any()
        .add(contains(OrganizationColumn::Name, value))
        .add(contains(OrganizationColumn::Alias, value))
}

fn listing_select(realm_id: Uuid, filter: &OrganizationFilter) -> Select<OrganizationEntity> {
    OrganizationEntity::find()
        .filter(OrganizationColumn::RealmId.eq(realm_id))
        .filter(within(OrganizationColumn::CreatedAt, &filter.created))
        .apply_if(filter.search.as_deref(), |select, value| {
            select.filter(search(value))
        })
        .apply_if(filter.name.as_deref(), |select, value| {
            select.filter(contains(OrganizationColumn::Name, value))
        })
        .apply_if(filter.alias.as_deref(), |select, value| {
            select.filter(contains(OrganizationColumn::Alias, value))
        })
        .apply_if(filter.domain.as_deref(), |select, value| {
            select.filter(contains(OrganizationColumn::Domain, value))
        })
        .apply_if(filter.description.as_deref(), |select, value| {
            select.filter(contains(OrganizationColumn::Description, value))
        })
        .apply_if(filter.enabled, |select, value| {
            select.filter(OrganizationColumn::Enabled.eq(value))
        })
        .apply_if(filter.has_domain, |select, present| {
            select.filter(if present {
                OrganizationColumn::Domain.is_not_null()
            } else {
                OrganizationColumn::Domain.is_null()
            })
        })
        .apply_if(filter.without_member, |select, user_id| {
            let joined = OrganizationMemberEntity::find()
                .select_only()
                .column(OrganizationMemberColumn::OrganizationId)
                .filter(OrganizationMemberColumn::UserId.eq(user_id))
                .into_query();
            select.filter(OrganizationColumn::Id.not_in_subquery(joined))
        })
        .apply_if(filter.ids.as_deref(), |select, ids| {
            select.filter(OrganizationColumn::Id.is_in(ids.iter().copied()))
        })
}

impl OrganizationRepository for PostgresOrganizationRepository {
    async fn create_organization(
        &self,
        params: CreateOrganizationParams,
    ) -> Result<Organization, CoreError> {
        let (_, timestamp) = generate_timestamp();
        let id = Uuid::new_v7(timestamp);
        let now = Utc::now().fixed_offset();

        let model = OrganizationEntity::insert(OrganizationActiveModel {
            id: Set(id),
            realm_id: Set(params.realm_id.into()),
            name: Set(params.name),
            alias: Set(params.alias),
            domain: Set(params.domain),
            redirect_url: Set(params.redirect_url),
            description: Set(params.description),
            enabled: Set(params.enabled),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .exec_with_returning(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to create organization: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model_to_domain(model))
    }

    async fn get_organization_by_id(
        &self,
        id: OrganizationId,
    ) -> Result<Option<Unscoped<Organization>>, CoreError> {
        let model = OrganizationEntity::find_by_id(id.as_uuid())
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get organization by id: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(model_to_domain).map(Unscoped::new))
    }

    async fn get_organization_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> Result<Option<Organization>, CoreError> {
        let model = OrganizationEntity::find()
            .filter(OrganizationColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .filter(OrganizationColumn::Alias.eq(alias))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get organization by realm and alias: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(model_to_domain))
    }

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<OrganizationFilter, OrganizationSortField>,
    ) -> Result<Page<Organization>, CoreError> {
        let select = listing_select(scope.id().into(), &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            error!("Failed to list organizations: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(Page::new(
            models.into_iter().map(model_to_domain).collect(),
            total,
            request.page,
            request.limit,
        ))
    }

    async fn update_organization(
        &self,
        organization: &Scoped<Organization>,
        params: UpdateOrganizationParams,
    ) -> Result<Organization, CoreError> {
        let id = organization.get().id;
        let now = Utc::now().fixed_offset();

        let mut active_model = OrganizationActiveModel {
            id: Set(id.as_uuid()),
            updated_at: Set(now),
            ..Default::default()
        };

        if let Some(name) = params.name {
            active_model.name = Set(name);
        }
        if let Some(alias) = params.alias {
            active_model.alias = Set(alias);
        }
        if params.domain.is_some() {
            active_model.domain = Set(params.domain);
        }
        if params.redirect_url.is_some() {
            active_model.redirect_url = Set(params.redirect_url);
        }
        if params.description.is_some() {
            active_model.description = Set(params.description);
        }
        if let Some(enabled) = params.enabled {
            active_model.enabled = Set(enabled);
        }

        let model = OrganizationEntity::update(active_model)
            .filter(OrganizationColumn::Id.eq(id.as_uuid()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to update organization: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model_to_domain(model))
    }

    async fn delete_organization(
        &self,
        organization: &Scoped<Organization>,
    ) -> Result<(), CoreError> {
        OrganizationEntity::delete_by_id(organization.get().id.as_uuid())
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete organization: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn exists_organization_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> Result<bool, CoreError> {
        let count: u64 = OrganizationEntity::find()
            .filter(OrganizationColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .filter(OrganizationColumn::Alias.eq(alias))
            .count(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to check organization alias existence: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(count > 0)
    }
}

#[cfg(test)]
mod listing_tests {
    use chrono::{TimeZone, Utc};
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;
    use crate::domain::common::pagination::DateRange;
    use ferriskey_organization::OrganizationFilter;

    #[test]
    fn an_unbounded_created_range_adds_no_predicate() {
        let sql = sql(&OrganizationFilter::default());
        assert!(!sql.contains(r#""organizations"."created_at" >"#), "{sql}");
        assert!(!sql.contains(r#""organizations"."created_at" <"#), "{sql}");
    }

    #[test]
    fn created_range_bounds_the_creation_date() {
        let sql = sql(&OrganizationFilter {
            created: DateRange::new(
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).single(),
            ),
            ..OrganizationFilter::default()
        });
        assert!(
            sql.contains(r#""organizations"."created_at" >= '2026-01-01 00:00:00.000000 +00:00'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""organizations"."created_at" < '2026-02-01 00:00:00.000000 +00:00'"#),
            "{sql}"
        );
    }

    #[test]
    fn description_is_an_escaped_contains_match() {
        let sql = sql(&OrganizationFilter {
            description: Some("d%".to_string()),
            ..OrganizationFilter::default()
        });
        assert!(
            sql.contains(r#""organizations"."description" ILIKE E'%d\\%%'"#),
            "{sql}"
        );
    }

    fn sql(filter: &OrganizationFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&OrganizationFilter::default());
        assert!(
            sql.contains(r#""organizations"."realm_id" = '00000000-0000-0000-0000-000000000000'"#),
            "{sql}"
        );
    }

    #[test]
    fn text_filters_are_escaped_contains_matches() {
        let sql = sql(&OrganizationFilter {
            name: Some("a%".to_string()),
            alias: Some("b".to_string()),
            domain: Some("c_".to_string()),
            ..OrganizationFilter::default()
        });
        assert!(
            sql.contains(r#""organizations"."name" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""organizations"."alias" ILIKE '%b%'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""organizations"."domain" ILIKE E'%c\\_%'"#),
            "{sql}"
        );
    }

    #[test]
    fn search_matches_the_name_or_the_alias() {
        let sql = sql(&OrganizationFilter {
            search: Some("acme".to_string()),
            ..OrganizationFilter::default()
        });
        assert!(
            sql.contains(
                r#"(("organizations"."name" ILIKE '%acme%') OR ("organizations"."alias" ILIKE '%acme%'))"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn enabled_is_an_equality() {
        let sql = sql(&OrganizationFilter {
            enabled: Some(false),
            ..OrganizationFilter::default()
        });
        assert!(
            sql.contains(r#""organizations"."enabled" = FALSE"#),
            "{sql}"
        );
    }

    #[test]
    fn has_domain_tests_the_domain_for_null() {
        let with = sql(&OrganizationFilter {
            has_domain: Some(true),
            ..OrganizationFilter::default()
        });
        assert!(
            with.contains(r#""organizations"."domain" IS NOT NULL"#),
            "{with}"
        );
        let without = sql(&OrganizationFilter {
            has_domain: Some(false),
            ..OrganizationFilter::default()
        });
        assert!(
            without.contains(r#""organizations"."domain" IS NULL"#),
            "{without}"
        );
    }

    #[test]
    fn without_member_excludes_the_organizations_of_that_user() {
        let user_id = Uuid::from_u128(7);
        let sql = sql(&OrganizationFilter {
            without_member: Some(user_id),
            ..OrganizationFilter::default()
        });
        assert!(
            sql.contains(&format!(
                r#""organizations"."id" NOT IN (SELECT "organization_members"."organization_id" FROM "organization_members" WHERE "organization_members"."user_id" = '{user_id}')"#
            )),
            "{sql}"
        );
    }

    #[test]
    fn ids_filter_is_an_in_list() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let sql = sql(&OrganizationFilter {
            ids: Some(vec![first, second]),
            ..OrganizationFilter::default()
        });
        assert!(
            sql.contains(&format!(
                r#""organizations"."id" IN ('{first}', '{second}')"#
            )),
            "{sql}"
        );
    }
}
