use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    TransactionTrait,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::{entities::app_errors::CoreError, generate_uuid_v7},
        portal_theme::{
            entities::{PortalPageType, PortalTheme, PortalThemeConfig, PortalThemePages},
            ports::PortalThemeRepository,
        },
    },
    entity::portal_themes::{ActiveModel, Column, Entity, Model},
};

#[derive(Debug, Clone)]
pub struct PostgresPortalThemeRepository {
    pub db: DatabaseConnection,
}

impl PostgresPortalThemeRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
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
    }
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

    async fn list_by_realm(&self, realm_id: Uuid) -> Result<Vec<PortalTheme>, CoreError> {
        let models = Entity::find()
            .filter(Column::RealmId.eq(realm_id))
            .order_by_asc(Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("failed to list portal themes: {e}");
                CoreError::InternalServerError
            })?;

        models.into_iter().map(model_to_domain).collect()
    }

    async fn get_by_id(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
    ) -> Result<Option<PortalTheme>, CoreError> {
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

    async fn create(
        &self,
        realm_id: Uuid,
        name: String,
        layout_id: Option<Uuid>,
        config: PortalThemeConfig,
    ) -> Result<PortalTheme, CoreError> {
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
        realm_id: Uuid,
        theme_id: Uuid,
        name: String,
        layout_id: Option<Uuid>,
        config: PortalThemeConfig,
    ) -> Result<PortalTheme, CoreError> {
        let existing = Entity::find_by_id(theme_id)
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
        realm_id: Uuid,
        theme_id: Uuid,
        page_type: PortalPageType,
        tree: serde_json::Value,
    ) -> Result<PortalTheme, CoreError> {
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

        self.get_by_id(realm_id, theme_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn activate(&self, realm_id: Uuid, theme_id: Uuid) -> Result<(), CoreError> {
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

    async fn delete(&self, realm_id: Uuid, theme_id: Uuid) -> Result<(), CoreError> {
        let result = Entity::delete_many()
            .filter(Column::Id.eq(theme_id))
            .filter(Column::RealmId.eq(realm_id))
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
