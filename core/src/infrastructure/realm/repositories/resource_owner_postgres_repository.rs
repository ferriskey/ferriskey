use chrono::Utc;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, TransactionTrait,
};
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::{RealmId, ResourceOwner};
use crate::domain::realm::ports::ResourceOwnerRepository;
use crate::entity::realm_resource_owners::{ActiveModel, Column, Entity, Model};

impl From<Model> for ResourceOwner {
    fn from(model: Model) -> Self {
        Self {
            uri: model.uri,
            client_id: model.client_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresResourceOwnerRepository {
    pub db: DatabaseConnection,
}

impl PostgresResourceOwnerRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ResourceOwnerRepository for PostgresResourceOwnerRepository {
    async fn list_by_realm(&self, realm_id: RealmId) -> Result<Vec<ResourceOwner>, CoreError> {
        let realm_id: Uuid = realm_id.into();

        let rows = Entity::find()
            .filter(Column::RealmId.eq(realm_id))
            .order_by_asc(Column::Uri)
            .all(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(ResourceOwner::from).collect())
    }

    async fn replace_for_realm(
        &self,
        realm_id: RealmId,
        owners: &[ResourceOwner],
    ) -> Result<(), CoreError> {
        let realm_id: Uuid = realm_id.into();

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Entity::delete_many()
            .filter(Column::RealmId.eq(realm_id))
            .exec(&txn)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        if !owners.is_empty() {
            let now = Utc::now().fixed_offset();
            let rows = owners.iter().map(|owner| ActiveModel {
                realm_id: Set(realm_id),
                uri: Set(owner.uri.clone()),
                client_id: Set(owner.client_id),
                created_at: Set(now),
            });

            Entity::insert_many(rows)
                .exec(&txn)
                .await
                .map_err(|e| CoreError::Database(e.to_string()))?;
        }

        txn.commit()
            .await
            .map_err(|e| CoreError::Database(e.to_string()))
    }

    async fn owns_any(
        &self,
        realm_id: RealmId,
        client_id: Uuid,
        uris: &[String],
    ) -> Result<bool, CoreError> {
        if uris.is_empty() {
            return Ok(false);
        }

        let realm_id: Uuid = realm_id.into();

        let owned = Entity::find()
            .filter(Column::RealmId.eq(realm_id))
            .filter(Column::ClientId.eq(client_id))
            .filter(Column::Uri.is_in(uris.iter().cloned()))
            .count(&self.db)
            .await
            .map_err(|e| CoreError::Database(e.to_string()))?;

        Ok(owned > 0)
    }
}
