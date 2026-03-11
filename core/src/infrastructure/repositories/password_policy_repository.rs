use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;
use crate::domain::{
    common::entities::app_errors::CoreError,
    password_policy::{
        entities::{PasswordPolicy, UpdatePasswordPolicy},
        ports::PasswordPolicyRepository,
    },
};
use chrono::{Utc, TimeZone};

impl From<crate::entity::password_policy::Model> for PasswordPolicy {
    fn from(model: crate::entity::password_policy::Model) -> Self {
        Self {
            id: model.id,
            realm_id: model.realm_id.into(),
            min_length: model.min_length,
            require_uppercase: model.require_uppercase,
            require_lowercase: model.require_lowercase,
            require_number: model.require_number,
            require_special: model.require_special,
            max_age_days: model.max_age_days,
            created_at: Utc.from_utc_datetime(&model.created_at.naive_utc()),
            updated_at: Utc.from_utc_datetime(&model.updated_at.naive_utc()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresPasswordPolicyRepository {
    db: DatabaseConnection,
}

impl PostgresPasswordPolicyRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl PasswordPolicyRepository for PostgresPasswordPolicyRepository {
    async fn find_by_realm_id(&self, realm_id: Uuid) -> Result<Option<PasswordPolicy>, CoreError> {
        let model = crate::entity::password_policy::Entity::find()
            .filter(crate::entity::password_policy::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch password policy: {:?}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(Into::into))
    }

    async fn upsert(&self, realm_id: Uuid, update: UpdatePasswordPolicy) -> Result<PasswordPolicy, CoreError> {
        let existing = crate::entity::password_policy::Entity::find()
            .filter(crate::entity::password_policy::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch existing policy during upsert: {:?}", e);
                CoreError::InternalServerError
            })?;

        let now = Utc::now();

        let model = if let Some(policy) = existing {
            // Update
            let mut active_model: crate::entity::password_policy::ActiveModel = policy.into();
            
            if let Some(min_length) = update.min_length {
                active_model.min_length = Set(min_length);
            }
            if let Some(require_uppercase) = update.require_uppercase {
                active_model.require_uppercase = Set(require_uppercase);
            }
            if let Some(require_lowercase) = update.require_lowercase {
                active_model.require_lowercase = Set(require_lowercase);
            }
            if let Some(require_number) = update.require_number {
                active_model.require_number = Set(require_number);
            }
            if let Some(require_special) = update.require_special {
                active_model.require_special = Set(require_special);
            }
            if let Some(max_age_days) = update.max_age_days {
                active_model.max_age_days = Set(Some(max_age_days));
            }
            
            active_model.updated_at = Set(now.fixed_offset());
            active_model.update(&self.db).await
        } else {
            // Insert
            let active_model = crate::entity::password_policy::ActiveModel {
                id: Set(Uuid::now_v7()),
                realm_id: Set(realm_id),
                min_length: Set(update.min_length.unwrap_or(8)),
                require_uppercase: Set(update.require_uppercase.unwrap_or(false)),
                require_lowercase: Set(update.require_lowercase.unwrap_or(false)),
                require_number: Set(update.require_number.unwrap_or(false)),
                require_special: Set(update.require_special.unwrap_or(false)),
                max_age_days: Set(update.max_age_days),
                created_at: Set(now.fixed_offset()),
                updated_at: Set(now.fixed_offset()),
            };
            active_model.insert(&self.db).await
        }
        .map_err(|e| {
            tracing::error!("Failed to upsert password policy: {:?}", e);
            CoreError::InternalServerError
        })?;

        Ok(model.into())
    }
}
