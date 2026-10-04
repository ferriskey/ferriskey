use std::collections::HashMap;

use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect,
    QueryTrait, Select, TransactionTrait,
    sea_query::{Expr, Query, SimpleExpr},
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::{
            entities::app_errors::CoreError,
            generate_uuid_v7,
            pagination::{Page, PageRequest},
        },
        portal_layouts::{
            entities::{
                PortalLayout, PortalLayoutFilter, PortalLayoutListItem, PortalLayoutSortField,
            },
            ports::PortalLayoutsRepository,
        },
        realm::entities::{RealmScope, Scoped, Unscoped},
    },
    entity::{
        portal_layouts::{ActiveModel, Column, Entity, Model},
        portal_themes,
    },
    infrastructure::pagination::{SortColumn, contains, paginate},
};

#[derive(Debug, Clone)]
pub struct PostgresPortalLayoutsRepository {
    pub db: DatabaseConnection,
}

impl PostgresPortalLayoutsRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<Model> for PortalLayout {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            realm_id: model.realm_id.into(),
            name: model.name,
            tree: model.tree,
            is_default: model.is_default,
            created_at: Utc.from_utc_datetime(&model.created_at),
            updated_at: Utc.from_utc_datetime(&model.updated_at),
        }
    }
}

impl SortColumn<Entity> for PortalLayoutSortField {
    fn column(&self) -> Column {
        match self {
            PortalLayoutSortField::Name => Column::Name,
            PortalLayoutSortField::CreatedAt => Column::CreatedAt,
            PortalLayoutSortField::UpdatedAt => Column::UpdatedAt,
        }
    }
}

fn used_by_a_theme(realm_id: Uuid) -> SimpleExpr {
    Expr::exists(
        Query::select()
            .expr(Expr::val(1))
            .from(portal_themes::Entity)
            .and_where(
                Expr::col((portal_themes::Entity, portal_themes::Column::LayoutId))
                    .equals((Entity, Column::Id)),
            )
            .and_where(
                Expr::col((portal_themes::Entity, portal_themes::Column::RealmId)).eq(realm_id),
            )
            .to_owned(),
    )
}

fn listing_select(realm_id: Uuid, filter: &PortalLayoutFilter) -> Select<Entity> {
    Entity::find()
        .filter(Column::RealmId.eq(realm_id))
        .apply_if(filter.name.as_deref(), |select, value| {
            select.filter(contains(Column::Name, value))
        })
        .apply_if(filter.is_default, |select, value| {
            select.filter(Column::IsDefault.eq(value))
        })
        .apply_if(filter.in_use, |select, in_use| {
            select.filter(if in_use {
                used_by_a_theme(realm_id)
            } else {
                used_by_a_theme(realm_id).not()
            })
        })
        .apply_if(filter.ids.as_deref(), |select, ids| {
            select.filter(Column::Id.is_in(ids.iter().copied()))
        })
}

fn theme_count_select(realm_id: Uuid, layout_ids: Vec<Uuid>) -> Select<portal_themes::Entity> {
    portal_themes::Entity::find()
        .select_only()
        .column(portal_themes::Column::LayoutId)
        .column_as(portal_themes::Column::Id.count(), "theme_count")
        .filter(portal_themes::Column::RealmId.eq(realm_id))
        .filter(portal_themes::Column::LayoutId.is_in(layout_ids))
        .group_by(portal_themes::Column::LayoutId)
}

impl PostgresPortalLayoutsRepository {
    async fn theme_counts(
        &self,
        realm_id: Uuid,
        layout_ids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, u64>, CoreError> {
        if layout_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let rows: Vec<(Option<Uuid>, i64)> = theme_count_select(realm_id, layout_ids)
            .into_tuple()
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("failed to count themes per portal layout: {e}");
                CoreError::InternalServerError
            })?;

        Ok(rows
            .into_iter()
            .filter_map(|(layout, count)| Some((layout?, u64::try_from(count).ok()?)))
            .collect())
    }
}

impl PortalLayoutsRepository for PostgresPortalLayoutsRepository {
    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<PortalLayoutFilter, PortalLayoutSortField>,
    ) -> Result<Page<PortalLayoutListItem>, CoreError> {
        let realm_id: Uuid = scope.id().into();
        let (models, total) =
            paginate(&self.db, listing_select(realm_id, &request.filter), request)
                .await
                .map_err(|e| {
                    error!("failed to list portal layouts: {e}");
                    CoreError::InternalServerError
                })?;
        let counts = self
            .theme_counts(realm_id, models.iter().map(|model| model.id).collect())
            .await?;

        Ok(Page::new(
            models
                .into_iter()
                .map(|model| PortalLayoutListItem {
                    theme_count: counts.get(&model.id).copied().unwrap_or_default(),
                    layout: PortalLayout::from(model),
                })
                .collect(),
            total,
            request.page,
            request.limit,
        ))
    }

    async fn get_by_id(
        &self,
        realm_id: Uuid,
        layout_id: Uuid,
    ) -> Result<Option<Unscoped<PortalLayout>>, CoreError> {
        let model = Entity::find_by_id(layout_id)
            .filter(Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch portal layout: {e}");
                CoreError::InternalServerError
            })?;

        Ok(model.map(|model| Unscoped::new(PortalLayout::from(model))))
    }

    async fn get_default(&self, realm_id: Uuid) -> Result<Option<PortalLayout>, CoreError> {
        let model = Entity::find()
            .filter(Column::RealmId.eq(realm_id))
            .filter(Column::IsDefault.eq(true))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch default portal layout: {e}");
                CoreError::InternalServerError
            })?;

        Ok(model.map(PortalLayout::from))
    }

    async fn create(
        &self,
        realm_id: Uuid,
        name: String,
        tree: serde_json::Value,
        is_default: bool,
    ) -> Result<PortalLayout, CoreError> {
        let now = Utc::now().naive_utc();
        let model = ActiveModel {
            id: Set(generate_uuid_v7()),
            realm_id: Set(realm_id),
            name: Set(name),
            tree: Set(tree),
            is_default: Set(is_default),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let inserted = Entity::insert(model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("failed to create portal layout: {e}");
                CoreError::InternalServerError
            })?;

        Ok(inserted.into())
    }

    async fn update(
        &self,
        layout: &Scoped<PortalLayout>,
        name: String,
        tree: serde_json::Value,
    ) -> Result<PortalLayout, CoreError> {
        let existing = Entity::find_by_id(layout.get().id)
            .filter(Column::RealmId.eq::<Uuid>(layout.get().realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch portal layout for update: {e}");
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active: ActiveModel = existing.into();
        active.name = Set(name);
        active.tree = Set(tree);
        active.updated_at = Set(Utc::now().naive_utc());

        let updated = Entity::update(active).exec(&self.db).await.map_err(|e| {
            error!("failed to update portal layout: {e}");
            CoreError::InternalServerError
        })?;

        Ok(updated.into())
    }

    async fn set_default(&self, layout: &Scoped<PortalLayout>) -> Result<PortalLayout, CoreError> {
        let realm_id: Uuid = layout.get().realm_id.into();
        let layout_id = layout.get().id;

        let txn = self.db.begin().await.map_err(|e| {
            error!("failed to begin set_default transaction: {e}");
            CoreError::InternalServerError
        })?;

        // Confirm the target layout exists in this realm.
        let target = Entity::find_by_id(layout_id)
            .filter(Column::RealmId.eq(realm_id))
            .one(&txn)
            .await
            .map_err(|e| {
                error!("failed to fetch target layout: {e}");
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let now = Utc::now().naive_utc();

        // Clear any current default in the realm (no-op if none).
        Entity::update_many()
            .col_expr(Column::IsDefault, sea_orm::sea_query::Expr::value(false))
            .col_expr(Column::UpdatedAt, sea_orm::sea_query::Expr::value(now))
            .filter(Column::RealmId.eq(realm_id))
            .filter(Column::IsDefault.eq(true))
            .exec(&txn)
            .await
            .map_err(|e| {
                error!("failed to clear existing default layout: {e}");
                CoreError::InternalServerError
            })?;

        let mut active: ActiveModel = target.into();
        active.is_default = Set(true);
        active.updated_at = Set(now);

        let updated = Entity::update(active).exec(&txn).await.map_err(|e| {
            error!("failed to mark layout as default: {e}");
            CoreError::InternalServerError
        })?;

        txn.commit().await.map_err(|e| {
            error!("failed to commit set_default transaction: {e}");
            CoreError::InternalServerError
        })?;

        Ok(updated.into())
    }

    async fn delete(&self, layout: &Scoped<PortalLayout>) -> Result<(), CoreError> {
        let result = Entity::delete_many()
            .filter(Column::Id.eq(layout.get().id))
            .filter(Column::RealmId.eq::<Uuid>(layout.get().realm_id.into()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("failed to delete portal layout: {e}");
                CoreError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn is_used_by_themes(&self, layout: &Scoped<PortalLayout>) -> Result<bool, CoreError> {
        let count = portal_themes::Entity::find()
            .filter(portal_themes::Column::RealmId.eq::<Uuid>(layout.get().realm_id.into()))
            .filter(portal_themes::Column::LayoutId.eq(layout.get().id))
            .count(&self.db)
            .await
            .map_err(|e| {
                error!("failed to count themes using portal layout: {e}");
                CoreError::InternalServerError
            })?;

        Ok(count > 0)
    }

    async fn is_active_theme_layout(
        &self,
        layout: &Scoped<PortalLayout>,
    ) -> Result<bool, CoreError> {
        use crate::entity::realm_settings;

        let realm_id: Uuid = layout.get().realm_id.into();

        let settings = realm_settings::Entity::find()
            .filter(realm_settings::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch realm_settings for the active portal theme: {e}");
                CoreError::InternalServerError
            })?;

        let Some(theme_id) = settings.and_then(|settings| settings.portal_theme_id) else {
            return Ok(false);
        };

        let count = portal_themes::Entity::find()
            .filter(portal_themes::Column::Id.eq(theme_id))
            .filter(portal_themes::Column::RealmId.eq(realm_id))
            .filter(portal_themes::Column::LayoutId.eq(layout.get().id))
            .count(&self.db)
            .await
            .map_err(|e| {
                error!("failed to check the active theme layout: {e}");
                CoreError::InternalServerError
            })?;

        Ok(count > 0)
    }
}

#[cfg(test)]
mod listing_tests {
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::{listing_select, theme_count_select};
    use crate::domain::portal_layouts::entities::PortalLayoutFilter;

    fn sql(filter: &PortalLayoutFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&PortalLayoutFilter::default());
        assert!(
            sql.contains(r#""portal_layouts"."realm_id" = '00000000-0000-0000-0000-000000000000'"#),
            "{sql}"
        );
        assert!(!sql.contains("ILIKE"), "{sql}");
        assert!(!sql.contains("EXISTS"), "{sql}");
        assert!(!sql.contains(" IN "), "{sql}");
    }

    #[test]
    fn name_is_an_escaped_contains_match() {
        let sql = sql(&PortalLayoutFilter {
            name: Some("a%".to_string()),
            ..PortalLayoutFilter::default()
        });
        assert!(
            sql.contains(r#""portal_layouts"."name" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
    }

    #[test]
    fn is_default_is_an_equality() {
        let sql = sql(&PortalLayoutFilter {
            is_default: Some(false),
            ..PortalLayoutFilter::default()
        });
        assert!(
            sql.contains(r#""portal_layouts"."is_default" = FALSE"#),
            "{sql}"
        );
    }

    #[test]
    fn in_use_checks_for_a_theme_of_the_same_realm() {
        let sql = sql(&PortalLayoutFilter {
            in_use: Some(true),
            ..PortalLayoutFilter::default()
        });
        assert!(
            sql.contains(
                r#"EXISTS(SELECT 1 FROM "portal_themes" WHERE "portal_themes"."layout_id" = "portal_layouts"."id" AND "portal_themes"."realm_id" = '00000000-0000-0000-0000-000000000000')"#
            ),
            "{sql}"
        );
        assert!(!sql.contains("NOT"), "{sql}");
    }

    #[test]
    fn not_in_use_negates_the_theme_check() {
        let sql = sql(&PortalLayoutFilter {
            in_use: Some(false),
            ..PortalLayoutFilter::default()
        });
        assert!(
            sql.contains(r#"NOT EXISTS(SELECT 1 FROM "portal_themes""#),
            "{sql}"
        );
    }

    #[test]
    fn ids_filter_is_an_in_list() {
        let sql = sql(&PortalLayoutFilter {
            ids: Some(vec![Uuid::nil(), Uuid::max()]),
            ..PortalLayoutFilter::default()
        });
        assert!(
            sql.contains(r#""portal_layouts"."id" IN ('00000000-0000-0000-0000-000000000000', 'ffffffff-ffff-ffff-ffff-ffffffffffff')"#),
            "{sql}"
        );
    }

    #[test]
    fn theme_counts_are_one_grouped_query_bound_to_the_realm() {
        let sql = theme_count_select(Uuid::nil(), vec![Uuid::max()])
            .build(DbBackend::Postgres)
            .to_string();
        assert_eq!(
            sql,
            r#"SELECT "portal_themes"."layout_id", COUNT("portal_themes"."id") AS "theme_count" FROM "portal_themes" WHERE "portal_themes"."realm_id" = '00000000-0000-0000-0000-000000000000' AND "portal_themes"."layout_id" IN ('ffffffff-ffff-ffff-ffff-ffffffffffff') GROUP BY "portal_themes"."layout_id""#
        );
    }
}
