use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    sea_query::OnConflict,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::{entities::app_errors::CoreError, generate_timestamp},
        portal::{entities::PortalConfig, ports::PortalRepository},
    },
    entity::realm_portal_configs::{
        ActiveModel as PortalConfigActiveModel, Column as PortalConfigColumn,
        Entity as PortalConfigEntity,
    },
};

#[derive(Debug, Clone)]
pub struct PostgresPortalRepository {
    pub db: DatabaseConnection,
}

impl PostgresPortalRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl PortalRepository for PostgresPortalRepository {
    async fn get_by_realm(&self, realm_id: Uuid) -> Result<Option<PortalConfig>, CoreError> {
        PortalConfigEntity::find()
            .filter(PortalConfigColumn::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map(|model| model.map(PortalConfig::from))
            .map_err(|e| {
                error!("Failed to get portal config: {}", e);
                CoreError::InternalServerError
            })
    }

    async fn get_active_by_realm(&self, realm_id: Uuid) -> Result<Option<PortalConfig>, CoreError> {
        PortalConfigEntity::find()
            .filter(PortalConfigColumn::RealmId.eq(realm_id))
            .filter(PortalConfigColumn::IsActive.eq(true))
            .one(&self.db)
            .await
            .map(|model| model.map(PortalConfig::from))
            .map_err(|e| {
                error!("Failed to get active portal config: {}", e);
                CoreError::InternalServerError
            })
    }

    async fn upsert(
        &self,
        realm_id: Uuid,
        layout: serde_json::Value,
    ) -> Result<PortalConfig, CoreError> {
        let (_, timestamp) = generate_timestamp();
        let id = Uuid::new_v7(timestamp);
        let now = chrono::Utc::now().naive_utc();

        let model = PortalConfigActiveModel {
            id: Set(id),
            realm_id: Set(realm_id),
            is_active: Set(false),
            layout: Set(layout.clone()),
            created_at: Set(now),
            updated_at: Set(now),
        };

        PortalConfigEntity::insert(model)
            .on_conflict(
                OnConflict::column(PortalConfigColumn::RealmId)
                    .update_columns([PortalConfigColumn::Layout, PortalConfigColumn::UpdatedAt])
                    .to_owned(),
            )
            .exec_with_returning(&self.db)
            .await
            .map(PortalConfig::from)
            .map_err(|e| {
                error!("Failed to upsert portal config: {}", e);
                CoreError::InternalServerError
            })
    }

    async fn set_active(&self, realm_id: Uuid, is_active: bool) -> Result<PortalConfig, CoreError> {
        let existing = PortalConfigEntity::find()
            .filter(PortalConfigColumn::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to find portal config for update: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::PortalConfigNotFound)?;

        let mut active: PortalConfigActiveModel = existing.into();
        active.is_active = Set(is_active);
        active.updated_at = Set(chrono::Utc::now().naive_utc());

        active
            .update(&self.db)
            .await
            .map(PortalConfig::from)
            .map_err(|e| {
                error!("Failed to update portal config active state: {}", e);
                CoreError::InternalServerError
            })
    }

    async fn delete(&self, realm_id: Uuid) -> Result<(), CoreError> {
        PortalConfigEntity::delete_many()
            .filter(PortalConfigColumn::RealmId.eq(realm_id))
            .exec(&self.db)
            .await
            .map(|_| ())
            .map_err(|e| {
                error!("Failed to delete portal config: {}", e);
                CoreError::InternalServerError
            })
    }
}
