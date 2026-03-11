use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveValue};
use uuid::Uuid;
use crate::domain::{
    common::entities::app_errors::CoreError,
    password_policy::{
        entities::{PasswordPolicy, UpdatePasswordPolicy},
        ports::PasswordPolicyRepository,
    },
};
use crate::entity::password_policy::{Entity as PasswordPolicyEntity, Column, ActiveModel};
use chrono::{DateTime, Utc, FixedOffset};

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
        let model = PasswordPolicyEntity::find()
            .filter(Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch password policy: {:?}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(|m| PasswordPolicy {
            id: m.id,
            realm_id: m.realm_id.into(),
            min_length: m.min_length,
            require_uppercase: m.require_uppercase,
            require_lowercase: m.require_lowercase,
            require_number: m.require_number,
            require_special: m.require_special,
            max_age_days: m.max_age_days,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
        }))
    }

    async fn upsert(&self, realm_id: Uuid, update: UpdatePasswordPolicy) -> Result<PasswordPolicy, CoreError> {
        let now: DateTime<FixedOffset> = Utc::now().into();

        let mut active_model = ActiveModel {
            realm_id: ActiveValue::Set(realm_id),
            updated_at: ActiveValue::Set(now),
            ..Default::default()
        };

        if let Some(min_length) = update.min_length {
            active_model.min_length = ActiveValue::Set(min_length);
        }
        if let Some(require_uppercase) = update.require_uppercase {
            active_model.require_uppercase = ActiveValue::Set(require_uppercase);
        }
        if let Some(require_lowercase) = update.require_lowercase {
            active_model.require_lowercase = ActiveValue::Set(require_lowercase);
        }
        if let Some(require_number) = update.require_number {
            active_model.require_number = ActiveValue::Set(require_number);
        }
        if let Some(require_special) = update.require_special {
            active_model.require_special = ActiveValue::Set(require_special);
        }
        
        // Tri-state max_age_days:
        // None = omitted (don't set)
        // Some(None) = set to null
        // Some(Some(v)) = set to value
        match update.max_age_days {
            Some(val) => active_model.max_age_days = ActiveValue::Set(val),
            None => {}
        }

        // Atomic upsert via on_conflict
        let model = PasswordPolicyEntity::insert(active_model)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(Column::RealmId)
                    .update_columns(vec![
                        Column::MinLength,
                        Column::RequireUppercase,
                        Column::RequireLowercase,
                        Column::RequireNumber,
                        Column::RequireSpecial,
                        Column::MaxAgeDays,
                        Column::UpdatedAt,
                    ])
                    .to_owned()
            )
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to upsert password policy: {:?}", e);
                CoreError::InternalServerError
            })?;

        Ok(PasswordPolicy {
            id: model.id,
            realm_id: model.realm_id.into(),
            min_length: model.min_length,
            require_uppercase: model.require_uppercase,
            require_lowercase: model.require_lowercase,
            require_number: model.require_number,
            require_special: model.require_special,
            max_age_days: model.max_age_days,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        })
    }
}
