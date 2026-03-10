use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        realm::{
            entities::{RealmId, SmtpConfig},
            ports::SmtpConfigRepository,
        },
    },
    entity::smtp_configs::{ActiveModel, Entity as SmtpConfigEntity},
};

#[derive(Debug, Clone)]
pub struct PostgresSmtpConfigRepository {
    pub db: DatabaseConnection,
}

impl PostgresSmtpConfigRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl SmtpConfigRepository for PostgresSmtpConfigRepository {
    async fn get_by_realm_id(&self, realm_id: RealmId) -> Result<Option<SmtpConfig>, CoreError> {
        let result = SmtpConfigEntity::find()
            .filter(crate::entity::smtp_configs::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?
            .map(SmtpConfig::from);

        Ok(result)
    }

    async fn upsert(&self, config: &SmtpConfig) -> Result<SmtpConfig, CoreError> {
        let now = chrono::Utc::now().fixed_offset();

        let existing = SmtpConfigEntity::find()
            .filter(crate::entity::smtp_configs::Column::RealmId.eq(config.realm_id))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        let model = if let Some(existing) = existing {
            let mut active: ActiveModel = existing.into();
            active.host = Set(config.host.clone());
            active.port = Set(config.port as i32);
            active.username = Set(config.username.clone());
            active.password = Set(config.password.clone());
            active.from_email = Set(config.from_email.clone());
            active.from_name = Set(config.from_name.clone());
            active.encryption = Set(config.encryption.as_str().to_string());
            active.updated_at = Set(now);

            active
                .update(&self.db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
        } else {
            let id = crate::domain::common::generate_uuid_v7();
            let active = ActiveModel {
                id: Set(id),
                realm_id: Set(config.realm_id),
                host: Set(config.host.clone()),
                port: Set(config.port as i32),
                username: Set(config.username.clone()),
                password: Set(config.password.clone()),
                from_email: Set(config.from_email.clone()),
                from_name: Set(config.from_name.clone()),
                encryption: Set(config.encryption.as_str().to_string()),
                created_at: Set(now),
                updated_at: Set(now),
            };

            active
                .insert(&self.db)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?
        };

        Ok(SmtpConfig::from(model))
    }

    async fn delete_by_realm_id(&self, realm_id: RealmId) -> Result<(), CoreError> {
        SmtpConfigEntity::delete_many()
            .filter(crate::entity::smtp_configs::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(())
    }
}
