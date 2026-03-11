use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};

use ferriskey_migrate::{
    entities::MigrationRecord, error::MigrationError, ports::MigrationRepository,
};

use crate::entity::data_migrations;

#[derive(Clone, Debug)]
pub struct PostgresMigrationRepository {
    db: DatabaseConnection,
}

impl PostgresMigrationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl MigrationRepository for PostgresMigrationRepository {
    async fn find_applied_versions(&self) -> Result<Vec<u64>, MigrationError> {
        let rows = data_migrations::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| MigrationError::LoadFailed(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.version as u64).collect())
    }

    async fn record_applied(&self, record: MigrationRecord) -> Result<(), MigrationError> {
        let model = data_migrations::ActiveModel {
            version: Set(record.version as i64),
            name: Set(record.name),
            applied_at: Set(record.applied_at.into()),
        };

        model
            .insert(&self.db)
            .await
            .map_err(|e| MigrationError::RecordFailed {
                version: record.version,
                reason: e.to_string(),
            })?;

        Ok(())
    }
}
