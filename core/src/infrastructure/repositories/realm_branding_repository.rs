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
        realm_branding::{
            entities::{BrandingConfig, RealmBranding},
            ports::RealmBrandingRepository,
        },
    },
    entity::realm_branding::{
        ActiveModel as RealmBrandingActiveModel, Column as RealmBrandingColumn,
        Entity as RealmBrandingEntity, Model as RealmBrandingModel,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresRealmBrandingRepository {
    pub db: DatabaseConnection,
}

impl PostgresRealmBrandingRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn model_to_domain(model: RealmBrandingModel) -> Result<RealmBranding, CoreError> {
    let config: BrandingConfig = serde_json::from_value(model.config).map_err(|e| {
        error!("failed to deserialize branding config: {e}");
        CoreError::InternalServerError
    })?;

    Ok(RealmBranding {
        id: model.id,
        realm_id: model.realm_id.into(),
        config,
        created_at: Utc.from_utc_datetime(&model.created_at),
        updated_at: Utc.from_utc_datetime(&model.updated_at),
    })
}

impl RealmBrandingRepository for PostgresRealmBrandingRepository {
    async fn get_by_realm(&self, realm_id: Uuid) -> Result<Option<RealmBranding>, CoreError> {
        let model = RealmBrandingEntity::find()
            .filter(RealmBrandingColumn::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch realm branding: {e}");
                CoreError::InternalServerError
            })?;

        model.map(model_to_domain).transpose()
    }

    async fn upsert(
        &self,
        realm_id: Uuid,
        config: BrandingConfig,
    ) -> Result<RealmBranding, CoreError> {
        let now = Utc::now().naive_utc();
        let config_json = serde_json::to_value(&config).map_err(|e| {
            error!("failed to serialize branding config: {e}");
            CoreError::InternalServerError
        })?;

        let model = RealmBrandingActiveModel {
            id: Set(generate_uuid_v7()),
            realm_id: Set(realm_id),
            config: Set(config_json),
            created_at: Set(now),
            updated_at: Set(now),
        };

        RealmBrandingEntity::insert(model)
            .on_conflict(
                OnConflict::column(RealmBrandingColumn::RealmId)
                    .update_columns([RealmBrandingColumn::Config, RealmBrandingColumn::UpdatedAt])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("failed to upsert realm branding: {e}");
                CoreError::InternalServerError
            })?;

        let stored = RealmBrandingEntity::find()
            .filter(RealmBrandingColumn::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch upserted realm branding: {e}");
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::InternalServerError)?;

        model_to_domain(stored)
    }
}
