use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    sea_query::OnConflict,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::{entities::app_errors::CoreError, generate_uuid_v7},
        portal_theme::{
            entities::{PortalTheme, PortalThemeConfig},
            ports::PortalThemeRepository,
        },
    },
    entity::realm_branding::{
        ActiveModel as PortalThemeActiveModel, Column as PortalThemeColumn,
        Entity as PortalThemeEntity, Model as PortalThemeModel,
    },
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

fn model_to_domain(model: PortalThemeModel) -> Result<PortalTheme, CoreError> {
    let config: PortalThemeConfig = serde_json::from_value(model.config).map_err(|e| {
        error!("failed to deserialize portal theme config: {e}");
        CoreError::InternalServerError
    })?;

    Ok(PortalTheme {
        id: model.id,
        realm_id: model.realm_id.into(),
        config,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    })
}

impl PortalThemeRepository for PostgresPortalThemeRepository {
    async fn get_by_realm(&self, realm_id: Uuid) -> Result<Option<PortalTheme>, CoreError> {
        let model = PortalThemeEntity::find()
            .filter(PortalThemeColumn::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch portal theme: {e}");
                CoreError::InternalServerError
            })?;

        model.map(model_to_domain).transpose()
    }

    async fn upsert(
        &self,
        realm_id: Uuid,
        config: PortalThemeConfig,
    ) -> Result<PortalTheme, CoreError> {
        let now = Utc::now().naive_utc();
        let config_json = serde_json::to_value(&config).map_err(|e| {
            error!("failed to serialize portal theme config: {e}");
            CoreError::InternalServerError
        })?;

        let model = PortalThemeActiveModel {
            id: Set(generate_uuid_v7()),
            realm_id: Set(realm_id),
            config: Set(config_json),
            created_at: Set(now),
            updated_at: Set(now),
        };

        PortalThemeEntity::insert(model)
            .on_conflict(
                OnConflict::column(PortalThemeColumn::RealmId)
                    .update_columns([PortalThemeColumn::Config, PortalThemeColumn::UpdatedAt])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("failed to upsert portal theme: {e}");
                CoreError::InternalServerError
            })?;

        let stored = PortalThemeEntity::find()
            .filter(PortalThemeColumn::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch upserted portal theme: {e}");
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::InternalServerError)?;

        model_to_domain(stored)
    }
}
