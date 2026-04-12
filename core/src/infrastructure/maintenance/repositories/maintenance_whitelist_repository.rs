use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use ferriskey_domain::maintenance::{
    entities::MaintenanceWhitelistEntry, ports::MaintenanceWhitelistRepository,
};

#[derive(Clone, Debug)]
pub struct PostgresMaintenanceWhitelistRepository {
    #[allow(dead_code)]
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
        _client_id: Uuid,
        _user_id: Uuid,
    ) -> Result<MaintenanceWhitelistEntry, CoreError> {
        todo!("M2: Infrastructure")
    }
    async fn add_role(
        &self,
        _client_id: Uuid,
        _role_id: Uuid,
    ) -> Result<MaintenanceWhitelistEntry, CoreError> {
        todo!("M2: Infrastructure")
    }
    async fn remove(&self, _entry_id: Uuid) -> Result<(), CoreError> {
        todo!("M2: Infrastructure")
    }
    async fn get_by_client_id(
        &self,
        _client_id: Uuid,
    ) -> Result<Vec<MaintenanceWhitelistEntry>, CoreError> {
        todo!("M2: Infrastructure")
    }
    async fn get_whitelisted_user_ids(&self, _client_id: Uuid) -> Result<Vec<Uuid>, CoreError> {
        todo!("M2: Infrastructure")
    }
    async fn get_whitelisted_role_ids(&self, _client_id: Uuid) -> Result<Vec<Uuid>, CoreError> {
        todo!("M2: Infrastructure")
    }
}
