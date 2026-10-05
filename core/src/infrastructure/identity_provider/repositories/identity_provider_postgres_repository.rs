use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QueryTrait, Select,
    sea_query::{Expr, SimpleExpr},
};
use tracing::instrument;
use uuid::Uuid;

use crate::domain::abyss::identity_provider::{
    CreateIdentityProviderRequest, IdentityProvider, IdentityProviderFilter,
    IdentityProviderHealth, IdentityProviderRepository, IdentityProviderSortField,
    UpdateIdentityProviderRequest,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_uuid_v7;
use crate::domain::common::pagination::{Page, PageRequest};
use crate::domain::realm::entities::{RealmId, RealmScope, Scoped, Unscoped};
use crate::entity::identity_providers::{ActiveModel, Column, Entity as IdentityProviderEntity};
use crate::infrastructure::pagination::{SortColumn, contains, paginate, within};

impl SortColumn<IdentityProviderEntity> for IdentityProviderSortField {
    fn column(&self) -> Column {
        match self {
            IdentityProviderSortField::Alias => Column::Alias,
            IdentityProviderSortField::DisplayName => Column::DisplayName,
            IdentityProviderSortField::ProviderId => Column::ProviderId,
            IdentityProviderSortField::Enabled => Column::Enabled,
            IdentityProviderSortField::CreatedAt => Column::CreatedAt,
            IdentityProviderSortField::UpdatedAt => Column::UpdatedAt,
        }
    }
}

const FALSY: &str = "'null'::jsonb, 'false'::jsonb, '0'::jsonb, '\"\"'::jsonb";
const FALSY_SECRET: &str = "'null'::jsonb, 'false'::jsonb, '0'::jsonb";

fn config_value_in(key: &str, values: &str) -> SimpleExpr {
    Expr::cust(format!(
        "COALESCE(\"identity_providers\".\"config\" -> '{key}', 'null'::jsonb) IN ({values})"
    ))
}

fn incomplete() -> Condition {
    Condition::any()
        .add(config_value_in("client_id", FALSY))
        .add(config_value_in("client_secret", FALSY_SECRET))
        .add(config_value_in("authorization_url", FALSY))
        .add(config_value_in("token_url", FALSY))
}

fn health(value: IdentityProviderHealth) -> Condition {
    let without_scopes = config_value_in("scopes", FALSY);
    match value {
        IdentityProviderHealth::Error => incomplete(),
        IdentityProviderHealth::Degraded => {
            Condition::all().add(incomplete().not()).add(without_scopes)
        }
        IdentityProviderHealth::Healthy => Condition::all()
            .add(incomplete().not())
            .add(without_scopes.not()),
    }
}

fn search(value: &str) -> Condition {
    Condition::any()
        .add(contains(Column::Alias, value))
        .add(contains(Column::DisplayName, value))
}

fn listing_select(
    realm_id: Uuid,
    filter: &IdentityProviderFilter,
) -> Select<IdentityProviderEntity> {
    IdentityProviderEntity::find()
        .filter(Column::RealmId.eq(realm_id))
        .filter(within(Column::CreatedAt, &filter.created))
        .apply_if(filter.search.as_deref(), |select, value| {
            select.filter(search(value))
        })
        .apply_if(filter.alias.as_deref(), |select, value| {
            select.filter(contains(Column::Alias, value))
        })
        .apply_if(filter.display_name.as_deref(), |select, value| {
            select.filter(contains(Column::DisplayName, value))
        })
        .apply_if(filter.provider_id.as_deref(), |select, value| {
            select.filter(Column::ProviderId.eq(value))
        })
        .apply_if(filter.enabled, |select, value| {
            select.filter(Column::Enabled.eq(value))
        })
        .apply_if(filter.health, |select, value| select.filter(health(value)))
}

/// PostgreSQL implementation of the IdentityProviderRepository trait
///
/// Provides data access operations for identity providers using SeaORM.
#[derive(Debug, Clone)]
pub struct PostgresIdentityProviderRepository {
    db: DatabaseConnection,
}

impl PostgresIdentityProviderRepository {
    /// Creates a new PostgresIdentityProviderRepository
    ///
    /// # Arguments
    /// * `db` - The database connection
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl IdentityProviderRepository for PostgresIdentityProviderRepository {
    #[instrument(skip(self, request), fields(realm_id = ?request.realm_id, alias = %request.alias))]
    async fn create_identity_provider(
        &self,
        request: CreateIdentityProviderRequest,
    ) -> Result<IdentityProvider, CoreError> {
        let now = chrono::Utc::now().fixed_offset();

        let payload = ActiveModel {
            id: Set(generate_uuid_v7()),
            realm_id: Set(request.realm_id.into()),
            alias: Set(request.alias),
            provider_id: Set(request.provider_id),
            enabled: Set(request.enabled),
            display_name: Set(request.display_name),
            first_broker_login_flow_alias: Set(request.first_broker_login_flow_alias),
            post_broker_login_flow_alias: Set(request.post_broker_login_flow_alias),
            store_token: Set(request.store_token),
            add_read_token_role_on_create: Set(request.add_read_token_role_on_create),
            trust_email: Set(request.trust_email),
            link_only: Set(request.link_only),
            config: Set(request.config),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let identity_provider = payload.insert(&self.db).await.map_err(|e| {
            tracing::error!("Failed to create identity provider: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(identity_provider.into())
    }

    #[instrument(skip(self), fields(identity_provider_id = %id))]
    async fn get_identity_provider_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<Unscoped<IdentityProvider>>, CoreError> {
        let identity_provider = IdentityProviderEntity::find()
            .filter(Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get identity provider by id: {}", e);
                CoreError::InternalServerError
            })?
            .map(IdentityProvider::from)
            .map(Unscoped::new);

        Ok(identity_provider)
    }

    #[instrument(skip(self), fields(realm_id = ?realm_id, alias = %alias))]
    async fn get_identity_provider_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> Result<Option<IdentityProvider>, CoreError> {
        let identity_provider = IdentityProviderEntity::find()
            .filter(Column::RealmId.eq::<Uuid>(realm_id.into()))
            .filter(Column::Alias.eq(alias))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get identity provider by realm and alias: {}", e);
                CoreError::InternalServerError
            })?
            .map(IdentityProvider::from);

        Ok(identity_provider)
    }

    #[instrument(skip(self), fields(realm_id = ?realm_id))]
    async fn list_identity_providers_by_realm(
        &self,
        realm_id: RealmId,
        only_enabled: Option<bool>,
    ) -> Result<Vec<IdentityProvider>, CoreError> {
        let mut query =
            IdentityProviderEntity::find().filter(Column::RealmId.eq::<Uuid>(realm_id.into()));

        if only_enabled.unwrap_or(false) {
            query = query.filter(Column::Enabled.eq(true))
        }

        let identity_providers = query
            .order_by_asc(Column::Alias)
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to list identity providers by realm: {}", e);
                CoreError::InternalServerError
            })?;

        let identity_providers: Vec<IdentityProvider> = identity_providers
            .into_iter()
            .map(IdentityProvider::from)
            .collect();

        Ok(identity_providers)
    }

    #[instrument(skip(self, scope, request), fields(realm_id = ?scope.id()))]
    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<IdentityProviderFilter, IdentityProviderSortField>,
    ) -> Result<Page<IdentityProvider>, CoreError> {
        let select = listing_select(scope.id().into(), &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            tracing::error!("Failed to list identity providers: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(Page::new(
            models.into_iter().map(IdentityProvider::from).collect(),
            total,
            request.page,
            request.limit,
        ))
    }

    #[instrument(skip(self, request), fields(identity_provider_id = %provider.get().id))]
    async fn update_identity_provider(
        &self,
        provider: &Scoped<IdentityProvider>,
        request: UpdateIdentityProviderRequest,
    ) -> Result<IdentityProvider, CoreError> {
        let existing = IdentityProviderEntity::find()
            .filter(Column::Id.eq::<Uuid>(provider.get().id.into()))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find identity provider for update: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::ProviderNotFound)?;

        let resolved_config = request.resolve_config(&existing.config)?;

        let mut identity_provider: ActiveModel = existing.into();

        if let Some(enabled) = request.enabled {
            identity_provider.enabled = Set(enabled);
        }
        if let Some(display_name) = request.display_name {
            identity_provider.display_name = Set(Some(display_name));
        }
        if let Some(first_broker_login_flow_alias) = request.first_broker_login_flow_alias {
            identity_provider.first_broker_login_flow_alias =
                Set(Some(first_broker_login_flow_alias));
        }
        if let Some(post_broker_login_flow_alias) = request.post_broker_login_flow_alias {
            identity_provider.post_broker_login_flow_alias =
                Set(Some(post_broker_login_flow_alias));
        }
        if let Some(store_token) = request.store_token {
            identity_provider.store_token = Set(store_token);
        }
        if let Some(add_read_token_role_on_create) = request.add_read_token_role_on_create {
            identity_provider.add_read_token_role_on_create = Set(add_read_token_role_on_create);
        }
        if let Some(trust_email) = request.trust_email {
            identity_provider.trust_email = Set(trust_email);
        }
        if let Some(link_only) = request.link_only {
            identity_provider.link_only = Set(link_only);
        }
        if let Some(config) = resolved_config {
            identity_provider.config = Set(config);
        }

        identity_provider.updated_at = Set(chrono::Utc::now().fixed_offset());

        let updated = identity_provider.update(&self.db).await.map_err(|e| {
            tracing::error!("Failed to update identity provider: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(updated.into())
    }

    #[instrument(skip(self), fields(identity_provider_id = %provider.get().id))]
    async fn delete_identity_provider(
        &self,
        provider: &Scoped<IdentityProvider>,
    ) -> Result<(), CoreError> {
        let result = IdentityProviderEntity::delete_many()
            .filter(Column::Id.eq::<Uuid>(provider.get().id.into()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete identity provider: {}", e);
                CoreError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(CoreError::ProviderNotFound);
        }

        Ok(())
    }

    #[instrument(skip(self), fields(realm_id = ?realm_id, alias = %alias))]
    async fn exists_identity_provider_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> Result<bool, CoreError> {
        let count = IdentityProviderEntity::find()
            .filter(Column::RealmId.eq::<Uuid>(realm_id.into()))
            .filter(Column::Alias.eq(alias))
            .count(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to check identity provider existence: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;
    use crate::domain::abyss::identity_provider::{IdentityProviderFilter, IdentityProviderHealth};
    use crate::domain::common::pagination::DateRange;

    #[test]
    fn search_matches_the_alias_or_the_display_name() {
        let sql = sql(&IdentityProviderFilter {
            search: Some("a%".to_string()),
            ..IdentityProviderFilter::default()
        });
        assert!(
            sql.contains(
                r#"(("identity_providers"."alias" ILIKE E'%a\\%%') OR ("identity_providers"."display_name" ILIKE E'%a\\%%'))"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn an_unbounded_created_range_adds_no_predicate() {
        let sql = sql(&IdentityProviderFilter::default());
        assert!(
            !sql.contains(r#""identity_providers"."created_at" >"#),
            "{sql}"
        );
        assert!(
            !sql.contains(r#""identity_providers"."created_at" <"#),
            "{sql}"
        );
    }

    #[test]
    fn created_range_bounds_the_creation_date() {
        let sql = sql(&IdentityProviderFilter {
            created: DateRange::new(
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).single(),
            ),
            ..IdentityProviderFilter::default()
        });
        assert!(
            sql.contains(
                r#""identity_providers"."created_at" >= '2026-01-01 00:00:00.000000 +00:00'"#
            ),
            "{sql}"
        );
        assert!(
            sql.contains(
                r#""identity_providers"."created_at" < '2026-02-01 00:00:00.000000 +00:00'"#
            ),
            "{sql}"
        );
    }

    fn sql(filter: &IdentityProviderFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    fn health(value: IdentityProviderHealth) -> String {
        sql(&IdentityProviderFilter {
            health: Some(value),
            ..IdentityProviderFilter::default()
        })
    }

    const MISSING_CLIENT_ID: &str = r#"COALESCE("identity_providers"."config" -> 'client_id', 'null'::jsonb) IN ('null'::jsonb, 'false'::jsonb, '0'::jsonb, '""'::jsonb)"#;
    const MISSING_SECRET: &str = r#"COALESCE("identity_providers"."config" -> 'client_secret', 'null'::jsonb) IN ('null'::jsonb, 'false'::jsonb, '0'::jsonb)"#;
    const MISSING_SCOPES: &str = r#"COALESCE("identity_providers"."config" -> 'scopes', 'null'::jsonb) IN ('null'::jsonb, 'false'::jsonb, '0'::jsonb, '""'::jsonb)"#;

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&IdentityProviderFilter::default());
        assert!(
            sql.contains(
                r#""identity_providers"."realm_id" = '00000000-0000-0000-0000-000000000000'"#
            ),
            "{sql}"
        );
        assert!(!sql.contains("->"), "{sql}");
    }

    #[test]
    fn text_filters_are_escaped_contains_matches() {
        let sql = sql(&IdentityProviderFilter {
            alias: Some("a%".to_string()),
            display_name: Some("b".to_string()),
            ..IdentityProviderFilter::default()
        });
        assert!(
            sql.contains(r#""identity_providers"."alias" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""identity_providers"."display_name" ILIKE '%b%'"#),
            "{sql}"
        );
    }

    #[test]
    fn provider_type_and_enabled_are_exact_matches() {
        let sql = sql(&IdentityProviderFilter {
            provider_id: Some("oidc".to_string()),
            enabled: Some(false),
            ..IdentityProviderFilter::default()
        });
        assert!(
            sql.contains(r#""identity_providers"."provider_id" = 'oidc'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""identity_providers"."enabled" = FALSE"#),
            "{sql}"
        );
    }

    #[test]
    fn error_health_is_any_required_key_missing() {
        let sql = health(IdentityProviderHealth::Error);
        assert!(sql.contains(MISSING_CLIENT_ID), "{sql}");
        assert!(sql.contains(MISSING_SECRET), "{sql}");
        assert!(sql.contains(" OR "), "{sql}");
        assert!(!sql.contains("'scopes'"), "{sql}");
    }

    #[test]
    fn degraded_and_healthy_split_complete_configs_on_scopes() {
        let degraded = health(IdentityProviderHealth::Degraded);
        assert!(
            degraded.contains(&format!("AND (NOT (({MISSING_CLIENT_ID}) OR")),
            "{degraded}"
        );
        assert!(
            degraded.contains(&format!(")))) AND ({MISSING_SCOPES})")),
            "{degraded}"
        );

        let healthy = health(IdentityProviderHealth::Healthy);
        assert!(
            healthy.contains(&format!("AND (NOT (({MISSING_CLIENT_ID}) OR")),
            "{healthy}"
        );
        assert!(
            healthy.contains(&format!(")))) AND (NOT ({MISSING_SCOPES}))")),
            "{healthy}"
        );
    }
}
