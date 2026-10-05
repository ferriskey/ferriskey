use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QueryTrait, Select,
    TransactionTrait,
    sea_query::{Alias, Expr, Func, SimpleExpr},
};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    domain::{
        common::{
            entities::app_errors::CoreError,
            generate_uuid_v7,
            pagination::{Page, PageRequest},
        },
        portal_theme::{
            entities::{
                PortalPageType, PortalTheme, PortalThemeConfig, PortalThemeFilter,
                PortalThemePages, PortalThemeSortField,
            },
            ports::PortalThemeRepository,
            validation::REQUIRED_BLOCKS,
        },
        realm::entities::{RealmScope, Scoped, Unscoped},
    },
    entity::portal_themes::{ActiveModel, Column, Entity, Model},
    infrastructure::pagination::{SortColumn, contains, paginate, within_naive},
};

#[derive(Debug, Clone)]
pub struct PostgresPortalThemeRepository {
    pub db: DatabaseConnection,
}

impl PostgresPortalThemeRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn load(&self, realm_id: Uuid, theme_id: Uuid) -> Result<Option<PortalTheme>, CoreError> {
        let model = Entity::find_by_id(theme_id)
            .filter(Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch portal theme: {e}");
                CoreError::InternalServerError
            })?;

        model.map(model_to_domain).transpose()
    }

    async fn require_layout_of_realm(
        &self,
        realm_id: Uuid,
        layout_id: Option<Uuid>,
    ) -> Result<(), CoreError> {
        use crate::entity::portal_layouts as pl;

        let Some(layout_id) = layout_id else {
            return Ok(());
        };

        let owned = pl::Entity::find_by_id(layout_id)
            .filter(pl::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch the layout a portal theme names: {e}");
                CoreError::InternalServerError
            })?;

        if owned.is_none() {
            warn!(
                %layout_id,
                %realm_id,
                "Refused to bind a portal theme to a layout outside its realm"
            );
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}

fn model_to_domain(model: Model) -> Result<PortalTheme, CoreError> {
    let config: PortalThemeConfig = serde_json::from_value(model.design_tokens).map_err(|e| {
        error!("failed to deserialize portal theme design_tokens: {e}");
        CoreError::InternalServerError
    })?;

    let pages = PortalThemePages {
        login: model.page_login,
        register: model.page_register,
        totp: model.page_totp,
        forgot_password: model.page_forgot_password,
        reset_password: model.page_reset_password,
        magic_link_verify: model.page_magic_link_verify,
        magic_link_request: model.page_magic_link_request,
        verify_email: model.page_verify_email,
        email_verified: model.page_email_verified,
        totp_setup: model.page_totp_setup,
        device_verify: model.page_device_verify,
        device_verified: model.page_device_verified,
        consent: model.page_consent,
    };

    Ok(PortalTheme {
        id: model.id,
        realm_id: model.realm_id.into(),
        name: model.name,
        layout_id: model.layout_id,
        config,
        pages,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    })
}

fn page_column(page_type: PortalPageType) -> Column {
    match page_type {
        PortalPageType::Login => Column::PageLogin,
        PortalPageType::Register => Column::PageRegister,
        PortalPageType::Totp => Column::PageTotp,
        PortalPageType::ForgotPassword => Column::PageForgotPassword,
        PortalPageType::ResetPassword => Column::PageResetPassword,
        PortalPageType::MagicLinkVerify => Column::PageMagicLinkVerify,
        PortalPageType::MagicLinkRequest => Column::PageMagicLinkRequest,
        PortalPageType::VerifyEmail => Column::PageVerifyEmail,
        PortalPageType::EmailVerified => Column::PageEmailVerified,
        PortalPageType::TotpSetup => Column::PageTotpSetup,
        PortalPageType::DeviceVerify => Column::PageDeviceVerify,
        PortalPageType::DeviceVerified => Column::PageDeviceVerified,
        PortalPageType::Consent => Column::PageConsent,
    }
}

impl SortColumn<Entity> for PortalThemeSortField {
    fn column(&self) -> Column {
        match self {
            PortalThemeSortField::Name => Column::Name,
            PortalThemeSortField::CreatedAt => Column::CreatedAt,
            PortalThemeSortField::UpdatedAt => Column::UpdatedAt,
        }
    }
}

fn page_holds_block(page_type: PortalPageType, block: &str) -> SimpleExpr {
    Func::cust(Alias::new("jsonb_path_exists"))
        .arg(Expr::col((Entity, page_column(page_type))))
        .arg(
            Expr::val(format!("strict $.** ? (@.type == \"{block}\")"))
                .cast_as(Alias::new("jsonpath")),
        )
        .arg(Expr::val("{}").cast_as(Alias::new("jsonb")))
        .arg(Expr::val(true))
        .into()
}

fn every_required_block_present() -> Condition {
    REQUIRED_BLOCKS
        .iter()
        .flat_map(|(page_type, blocks)| {
            blocks
                .iter()
                .map(|block| page_holds_block(*page_type, block))
        })
        .fold(Condition::all(), Condition::add)
}

fn listing_select(realm_id: Uuid, filter: &PortalThemeFilter) -> Select<Entity> {
    Entity::find()
        .filter(Column::RealmId.eq(realm_id))
        .filter(within_naive(Column::CreatedAt, &filter.created))
        .apply_if(filter.search.as_deref(), |select, value| {
            select.filter(contains(Column::Name, value))
        })
        .apply_if(filter.name.as_deref(), |select, value| {
            select.filter(contains(Column::Name, value))
        })
        .apply_if(filter.layout_id, |select, value| {
            select.filter(Column::LayoutId.eq(value))
        })
        .apply_if(filter.activatable, |select, activatable| {
            select.filter(if activatable {
                every_required_block_present()
            } else {
                every_required_block_present().not()
            })
        })
}

fn config_to_json(config: &PortalThemeConfig) -> Result<serde_json::Value, CoreError> {
    serde_json::to_value(config).map_err(|e| {
        error!("failed to serialize portal theme design_tokens: {e}");
        CoreError::InternalServerError
    })
}

impl PortalThemeRepository for PostgresPortalThemeRepository {
    // ---------- Legacy single-theme-per-realm shims ----------
    //
    // These two methods target the realm's *active* theme (the one referenced
    // by `realm_settings.portal_theme_id`), creating one named "Default" on
    // first write. They keep the existing `/portal/theme` endpoint working
    // until PR4 swaps it out for the collection API.

    async fn get_by_realm(&self, realm_id: Uuid) -> Result<Option<PortalTheme>, CoreError> {
        self.get_active(realm_id).await
    }

    async fn upsert(
        &self,
        realm_id: Uuid,
        config: PortalThemeConfig,
    ) -> Result<PortalTheme, CoreError> {
        let now = Utc::now().naive_utc();
        let config_json = config_to_json(&config)?;

        if let Some(active) = self.get_active(realm_id).await? {
            let mut model: ActiveModel = Entity::find_by_id(active.id)
                .one(&self.db)
                .await
                .map_err(|e| {
                    error!("failed to fetch active portal theme for upsert: {e}");
                    CoreError::InternalServerError
                })?
                .ok_or(CoreError::InternalServerError)?
                .into();
            model.design_tokens = Set(config_json);
            model.updated_at = Set(now);
            let updated = Entity::update(model).exec(&self.db).await.map_err(|e| {
                error!("failed to update active portal theme: {e}");
                CoreError::InternalServerError
            })?;
            return model_to_domain(updated);
        }

        let created = self
            .create(realm_id, "Default".to_string(), None, config)
            .await?;
        set_realm_active_theme(&self.db, realm_id, Some(created.id)).await?;
        Ok(created)
    }

    // ---------- Collection API ----------

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<PortalThemeFilter, PortalThemeSortField>,
    ) -> Result<Page<PortalTheme>, CoreError> {
        let select = listing_select(scope.id().into(), &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            error!("failed to list portal themes: {e}");
            CoreError::InternalServerError
        })?;

        let themes = models
            .into_iter()
            .map(model_to_domain)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page::new(themes, total, request.page, request.limit))
    }

    async fn get_by_id(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
    ) -> Result<Option<Unscoped<PortalTheme>>, CoreError> {
        self.load(realm_id, theme_id)
            .await
            .map(|theme| theme.map(Unscoped::new))
    }

    async fn create(
        &self,
        realm_id: Uuid,
        name: String,
        layout_id: Option<Uuid>,
        config: PortalThemeConfig,
    ) -> Result<PortalTheme, CoreError> {
        self.require_layout_of_realm(realm_id, layout_id).await?;

        let now = Utc::now().naive_utc();
        let model = ActiveModel {
            id: Set(generate_uuid_v7()),
            realm_id: Set(realm_id),
            name: Set(name),
            layout_id: Set(layout_id),
            design_tokens: Set(config_to_json(&config)?),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let inserted = Entity::insert(model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("failed to create portal theme: {e}");
                CoreError::InternalServerError
            })?;

        model_to_domain(inserted)
    }

    async fn update_metadata(
        &self,
        theme: &Scoped<PortalTheme>,
        name: String,
        layout_id: Option<Uuid>,
        config: PortalThemeConfig,
    ) -> Result<PortalTheme, CoreError> {
        let realm_id: Uuid = theme.get().realm_id.into();
        self.require_layout_of_realm(realm_id, layout_id).await?;

        let existing = Entity::find_by_id(theme.get().id)
            .filter(Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch portal theme for update: {e}");
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active: ActiveModel = existing.into();
        active.name = Set(name);
        active.layout_id = Set(layout_id);
        active.design_tokens = Set(config_to_json(&config)?);
        active.updated_at = Set(Utc::now().naive_utc());

        let updated = Entity::update(active).exec(&self.db).await.map_err(|e| {
            error!("failed to update portal theme metadata: {e}");
            CoreError::InternalServerError
        })?;

        model_to_domain(updated)
    }

    async fn update_page(
        &self,
        theme: &Scoped<PortalTheme>,
        page_type: PortalPageType,
        tree: serde_json::Value,
    ) -> Result<PortalTheme, CoreError> {
        let realm_id: Uuid = theme.get().realm_id.into();
        let theme_id = theme.get().id;
        let existing = Entity::find_by_id(theme_id)
            .filter(Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch portal theme for page update: {e}");
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let now = Utc::now().naive_utc();
        Entity::update_many()
            .col_expr(
                page_column(page_type),
                sea_orm::sea_query::Expr::value(tree),
            )
            .col_expr(Column::UpdatedAt, sea_orm::sea_query::Expr::value(now))
            .filter(Column::Id.eq(existing.id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("failed to update portal theme page: {e}");
                CoreError::InternalServerError
            })?;

        self.load(realm_id, theme_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn activate(&self, theme: &Scoped<PortalTheme>) -> Result<(), CoreError> {
        let realm_id: Uuid = theme.get().realm_id.into();
        let theme_id = theme.get().id;
        let txn = self.db.begin().await.map_err(|e| {
            error!("failed to begin activate transaction: {e}");
            CoreError::InternalServerError
        })?;

        Entity::find_by_id(theme_id)
            .filter(Column::RealmId.eq(realm_id))
            .one(&txn)
            .await
            .map_err(|e| {
                error!("failed to fetch target theme for activate: {e}");
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        set_realm_active_theme(&txn, realm_id, Some(theme_id)).await?;

        txn.commit().await.map_err(|e| {
            error!("failed to commit activate transaction: {e}");
            CoreError::InternalServerError
        })?;

        Ok(())
    }

    async fn delete(&self, theme: &Scoped<PortalTheme>) -> Result<(), CoreError> {
        let result = Entity::delete_many()
            .filter(Column::Id.eq(theme.get().id))
            .filter(Column::RealmId.eq::<Uuid>(theme.get().realm_id.into()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("failed to delete portal theme: {e}");
                CoreError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn get_active(&self, realm_id: Uuid) -> Result<Option<PortalTheme>, CoreError> {
        let settings = crate::entity::realm_settings::Entity::find()
            .filter(crate::entity::realm_settings::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch realm_settings for active theme: {e}");
                CoreError::InternalServerError
            })?;

        let Some(theme_id) = settings.and_then(|s| s.portal_theme_id) else {
            return Ok(None);
        };

        let model = Entity::find_by_id(theme_id)
            .filter(Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch active portal theme: {e}");
                CoreError::InternalServerError
            })?;

        model.map(model_to_domain).transpose()
    }
}

/// Set `realm_settings.portal_theme_id` for the given realm. Accepts any
/// `ConnectionTrait` so callers can use either the DB pool or a transaction.
async fn set_realm_active_theme<C>(
    conn: &C,
    realm_id: Uuid,
    theme_id: Option<Uuid>,
) -> Result<(), CoreError>
where
    C: sea_orm::ConnectionTrait,
{
    use crate::entity::realm_settings as rs;

    let existing = rs::Entity::find()
        .filter(rs::Column::RealmId.eq(realm_id))
        .one(conn)
        .await
        .map_err(|e| {
            error!("failed to fetch realm_settings: {e}");
            CoreError::InternalServerError
        })?
        .ok_or(CoreError::InternalServerError)?;

    let mut active: rs::ActiveModel = existing.into();
    active.portal_theme_id = Set(theme_id);
    active.updated_at = Set(Utc::now().naive_utc());

    rs::Entity::update(active).exec(conn).await.map_err(|e| {
        error!("failed to update realm_settings.portal_theme_id: {e}");
        CoreError::InternalServerError
    })?;

    Ok(())
}

#[cfg(test)]
mod listing_tests {
    use chrono::{TimeZone, Utc};
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;
    use crate::domain::common::pagination::DateRange;
    use crate::domain::portal_theme::{entities::PortalThemeFilter, validation::REQUIRED_BLOCKS};

    #[test]
    fn search_is_an_escaped_contains_match_on_the_name() {
        let sql = sql(&PortalThemeFilter {
            search: Some("a%".to_string()),
            ..PortalThemeFilter::default()
        });
        assert!(
            sql.contains(r#""portal_themes"."name" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
    }

    #[test]
    fn an_unbounded_created_range_adds_no_predicate() {
        let sql = sql(&PortalThemeFilter::default());
        assert!(!sql.contains(r#""portal_themes"."created_at" >"#), "{sql}");
        assert!(!sql.contains(r#""portal_themes"."created_at" <"#), "{sql}");
    }

    #[test]
    fn created_range_bounds_the_creation_date() {
        let sql = sql(&PortalThemeFilter {
            created: DateRange::new(
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).single(),
            ),
            ..PortalThemeFilter::default()
        });
        assert!(
            sql.contains(r#""portal_themes"."created_at" >= '2026-01-01 00:00:00.000000'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""portal_themes"."created_at" < '2026-02-01 00:00:00.000000'"#),
            "{sql}"
        );
    }

    fn sql(filter: &PortalThemeFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&PortalThemeFilter::default());
        assert!(
            sql.contains(r#""portal_themes"."realm_id" = '00000000-0000-0000-0000-000000000000'"#),
            "{sql}"
        );
        assert!(!sql.contains("ILIKE"), "{sql}");
        assert!(!sql.contains("jsonb_path_exists"), "{sql}");
    }

    #[test]
    fn name_is_an_escaped_contains_match() {
        let sql = sql(&PortalThemeFilter {
            name: Some("a%".to_string()),
            ..PortalThemeFilter::default()
        });
        assert!(
            sql.contains(r#""portal_themes"."name" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
    }

    #[test]
    fn layout_is_an_equality() {
        let sql = sql(&PortalThemeFilter {
            layout_id: Some(Uuid::nil()),
            ..PortalThemeFilter::default()
        });
        assert!(
            sql.contains(r#""portal_themes"."layout_id" = '00000000-0000-0000-0000-000000000000'"#),
            "{sql}"
        );
    }

    #[test]
    fn activatable_requires_every_required_block_on_every_page() {
        let sql = sql(&PortalThemeFilter {
            activatable: Some(true),
            ..PortalThemeFilter::default()
        });
        let required: usize = REQUIRED_BLOCKS.iter().map(|(_, blocks)| blocks.len()).sum();
        assert_eq!(sql.matches("jsonb_path_exists").count(), required, "{sql}");
        assert!(
            sql.contains(
                r#"jsonb_path_exists("portal_themes"."page_login", CAST(E'strict $.** ? (@.type == \"email_input\")' AS jsonpath), CAST('{}' AS jsonb), TRUE)"#
            ),
            "{sql}"
        );
        assert!(sql.contains(r#""portal_themes"."page_consent""#), "{sql}");
        assert!(!sql.contains("NOT"), "{sql}");
    }

    #[test]
    fn not_activatable_negates_the_whole_requirement() {
        let sql = sql(&PortalThemeFilter {
            activatable: Some(false),
            ..PortalThemeFilter::default()
        });
        assert!(sql.contains("NOT (jsonb_path_exists("), "{sql}");
    }
}
