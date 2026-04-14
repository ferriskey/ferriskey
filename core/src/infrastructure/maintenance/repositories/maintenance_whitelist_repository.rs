use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::entity::client_maintenance_whitelist::{ActiveModel, Column, Entity as WhitelistEntity};
use ferriskey_domain::maintenance::{
    entities::MaintenanceWhitelistEntry, ports::MaintenanceWhitelistRepository,
};

#[derive(Clone, Debug)]
pub struct PostgresMaintenanceWhitelistRepository {
    pub db: DatabaseConnection,
}

impl PostgresMaintenanceWhitelistRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl MaintenanceWhitelistRepository for PostgresMaintenanceWhitelistRepository {
    async fn add_user(
        &self,
        client_id: Uuid,
        user_id: Uuid,
    ) -> Result<MaintenanceWhitelistEntry, CoreError> {
        let entry = MaintenanceWhitelistEntry::new_user(client_id, user_id);
        let model = ActiveModel {
            id: Set(entry.id),
            client_id: Set(entry.client_id),
            user_id: Set(entry.user_id),
            role_id: Set(None),
            created_at: Set(entry.created_at.into()),
        };
        model
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;
        Ok(entry)
    }

    async fn add_role(
        &self,
        client_id: Uuid,
        role_id: Uuid,
    ) -> Result<MaintenanceWhitelistEntry, CoreError> {
        let entry = MaintenanceWhitelistEntry::new_role(client_id, role_id);
        let model = ActiveModel {
            id: Set(entry.id),
            client_id: Set(entry.client_id),
            user_id: Set(None),
            role_id: Set(entry.role_id),
            created_at: Set(entry.created_at.into()),
        };
        model
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;
        Ok(entry)
    }

    async fn remove(&self, entry_id: Uuid) -> Result<(), CoreError> {
        let result = WhitelistEntity::delete_many()
            .filter(Column::Id.eq(entry_id))
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    async fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<MaintenanceWhitelistEntry>, CoreError> {
        let models = WhitelistEntity::find()
            .filter(Column::ClientId.eq(client_id))
            .all(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn get_whitelisted_user_ids(&self, client_id: Uuid) -> Result<Vec<Uuid>, CoreError> {
        let models = WhitelistEntity::find()
            .filter(Column::ClientId.eq(client_id))
            .filter(Column::UserId.is_not_null())
            .all(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(models.into_iter().filter_map(|m| m.user_id).collect())
    }

    async fn get_whitelisted_role_ids(&self, client_id: Uuid) -> Result<Vec<Uuid>, CoreError> {
        let models = WhitelistEntity::find()
            .filter(Column::ClientId.eq(client_id))
            .filter(Column::RoleId.is_not_null())
            .all(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(models.into_iter().filter_map(|m| m.role_id).collect())
    }
}

impl From<crate::entity::client_maintenance_whitelist::Model> for MaintenanceWhitelistEntry {
    fn from(model: crate::entity::client_maintenance_whitelist::Model) -> Self {
        Self {
            id: model.id,
            client_id: model.client_id,
            user_id: model.user_id,
            role_id: model.role_id,
            created_at: model.created_at.into(),
        }
    }
}
