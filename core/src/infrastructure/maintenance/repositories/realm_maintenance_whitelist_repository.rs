use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::entity::realm_maintenance_whitelist::{ActiveModel, Column, Entity as WhitelistEntity};
use ferriskey_domain::maintenance::{
    entities::RealmMaintenanceWhitelistEntry, ports::RealmMaintenanceWhitelistRepository,
};
use ferriskey_domain::realm::RealmId;

#[derive(Clone, Debug)]
pub struct PostgresRealmMaintenanceWhitelistRepository {
    pub db: DatabaseConnection,
}

impl PostgresRealmMaintenanceWhitelistRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl RealmMaintenanceWhitelistRepository for PostgresRealmMaintenanceWhitelistRepository {
    async fn add_user(
        &self,
        realm_id: RealmId,
        user_id: Uuid,
    ) -> Result<RealmMaintenanceWhitelistEntry, CoreError> {
        let entry = RealmMaintenanceWhitelistEntry::new_user(realm_id, user_id);
        let model = ActiveModel {
            id: Set(entry.id),
            realm_id: Set(realm_id.into()),
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
        realm_id: RealmId,
        role_id: Uuid,
    ) -> Result<RealmMaintenanceWhitelistEntry, CoreError> {
        let entry = RealmMaintenanceWhitelistEntry::new_role(realm_id, role_id);
        let model = ActiveModel {
            id: Set(entry.id),
            realm_id: Set(realm_id.into()),
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

    async fn get_by_realm_id(
        &self,
        realm_id: RealmId,
    ) -> Result<Vec<RealmMaintenanceWhitelistEntry>, CoreError> {
        let models = WhitelistEntity::find()
            .filter(Column::RealmId.eq::<Uuid>(realm_id.into()))
            .all(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(models.into_iter().map(Into::into).collect())
    }

    async fn get_whitelisted_user_ids(&self, realm_id: RealmId) -> Result<Vec<Uuid>, CoreError> {
        let models = WhitelistEntity::find()
            .filter(Column::RealmId.eq::<Uuid>(realm_id.into()))
            .filter(Column::UserId.is_not_null())
            .all(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(models.into_iter().filter_map(|m| m.user_id).collect())
    }

    async fn get_whitelisted_role_ids(&self, realm_id: RealmId) -> Result<Vec<Uuid>, CoreError> {
        let models = WhitelistEntity::find()
            .filter(Column::RealmId.eq::<Uuid>(realm_id.into()))
            .filter(Column::RoleId.is_not_null())
            .all(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(models.into_iter().filter_map(|m| m.role_id).collect())
    }
}

impl From<crate::entity::realm_maintenance_whitelist::Model> for RealmMaintenanceWhitelistEntry {
    fn from(model: crate::entity::realm_maintenance_whitelist::Model) -> Self {
        Self {
            id: model.id,
            realm_id: model.realm_id.into(),
            user_id: model.user_id,
            role_id: model.role_id,
            created_at: model.created_at.into(),
        }
    }
}
