use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use ferriskey_domain::maintenance::{
    entities::RealmMaintenanceWhitelistEntry, ports::RealmMaintenanceWhitelistRepository,
};
use ferriskey_domain::realm::RealmId;

#[derive(Clone, Debug)]
pub struct PostgresRealmMaintenanceWhitelistRepository {
    #[allow(dead_code)]
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
        _realm_id: RealmId,
        _user_id: Uuid,
    ) -> Result<RealmMaintenanceWhitelistEntry, CoreError> {
        todo!("M2: Infrastructure")
    }
    async fn add_role(
        &self,
        _realm_id: RealmId,
        _role_id: Uuid,
    ) -> Result<RealmMaintenanceWhitelistEntry, CoreError> {
        todo!("M2: Infrastructure")
    }
    async fn remove(&self, _entry_id: Uuid) -> Result<(), CoreError> {
        todo!("M2: Infrastructure")
    }
    async fn get_by_realm_id(
        &self,
        _realm_id: RealmId,
    ) -> Result<Vec<RealmMaintenanceWhitelistEntry>, CoreError> {
        todo!("M2: Infrastructure")
    }
    async fn get_whitelisted_user_ids(&self, _realm_id: RealmId) -> Result<Vec<Uuid>, CoreError> {
        todo!("M2: Infrastructure")
    }
    async fn get_whitelisted_role_ids(&self, _realm_id: RealmId) -> Result<Vec<Uuid>, CoreError> {
        todo!("M2: Infrastructure")
    }
}
