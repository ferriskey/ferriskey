use std::collections::HashMap;

use chrono::Utc;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    sea_query::OnConflict,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::{
    common::{entities::app_errors::CoreError, generate_uuid_v7},
    realm::entities::RealmId,
    user::{entities::UserAttribute, ports::UserAttributeRepository},
};

#[derive(Debug, Clone)]
pub struct PostgresUserAttributeRepository {
    pub db: DatabaseConnection,
}

impl PostgresUserAttributeRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl UserAttributeRepository for PostgresUserAttributeRepository {
    async fn list_by_user_id(&self, user_id: Uuid) -> Result<Vec<UserAttribute>, CoreError> {
        let models = crate::entity::user_attributes::Entity::find()
            .filter(crate::entity::user_attributes::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("error listing user attributes: {:?}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(UserAttribute::from).collect())
    }

    async fn upsert_many(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
        attributes: HashMap<String, String>,
    ) -> Result<Vec<UserAttribute>, CoreError> {
        if attributes.is_empty() {
            return self.list_by_user_id(user_id).await;
        }

        let now = Utc::now();
        let keys: Vec<String> = attributes.keys().cloned().collect();

        let models: Vec<crate::entity::user_attributes::ActiveModel> = attributes
            .into_iter()
            .map(|(k, v)| crate::entity::user_attributes::ActiveModel {
                id: Set(generate_uuid_v7()),
                user_id: Set(user_id),
                realm_id: Set(realm_id.into()),
                key: Set(k),
                value: Set(v),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
            })
            .collect();

        crate::entity::user_attributes::Entity::insert_many(models)
            .on_conflict(
                OnConflict::columns([
                    crate::entity::user_attributes::Column::UserId,
                    crate::entity::user_attributes::Column::Key,
                ])
                .update_columns([
                    crate::entity::user_attributes::Column::Value,
                    crate::entity::user_attributes::Column::UpdatedAt,
                ])
                .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("error upserting user attributes: {:?}", e);
                CoreError::InternalServerError
            })?;

        let updated = crate::entity::user_attributes::Entity::find()
            .filter(crate::entity::user_attributes::Column::UserId.eq(user_id))
            .filter(crate::entity::user_attributes::Column::Key.is_in(keys))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("error fetching upserted user attributes: {:?}", e);
                CoreError::InternalServerError
            })?;

        Ok(updated.into_iter().map(UserAttribute::from).collect())
    }

    async fn delete_by_key(&self, user_id: Uuid, key: String) -> Result<(), CoreError> {
        let result = crate::entity::user_attributes::Entity::delete_many()
            .filter(
                sea_orm::Condition::all()
                    .add(crate::entity::user_attributes::Column::UserId.eq(user_id))
                    .add(crate::entity::user_attributes::Column::Key.eq(key)),
            )
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("error deleting user attribute: {:?}", e);
                CoreError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}
