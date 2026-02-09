use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::aegis::entities::ClientScope;
use crate::domain::aegis::ports::ClientScopeRepository;
use crate::domain::aegis::value_objects::{CreateClientScopeRequest, UpdateClientScopeRequest};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_uuid_v7;
use crate::domain::realm::entities::RealmId;
use crate::entity::client_scopes;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PostgresClientScopeRepository {
    pub db: DatabaseConnection,
}

impl PostgresClientScopeRepository {
    #[allow(dead_code)]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ClientScopeRepository for PostgresClientScopeRepository {
    async fn create(&self, payload: CreateClientScopeRequest) -> Result<ClientScope, CoreError> {
        let now = Utc::now().naive_utc();

        let active_model = client_scopes::ActiveModel {
            id: Set(generate_uuid_v7()),
            realm_id: Set(payload.realm_id.into()),
            name: Set(payload.name),
            description: Set(payload.description),
            protocol: Set(payload.protocol),
            is_default: Set(payload.is_default),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let model = active_model.insert(&self.db).await.map_err(|e| {
            tracing::error!("Failed to insert client scope: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model.into())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<ClientScope>, CoreError> {
        let model = client_scopes::Entity::find()
            .filter(client_scopes::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get client scope by id: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(ClientScope::from))
    }

    async fn find_by_realm_id(&self, realm_id: RealmId) -> Result<Vec<ClientScope>, CoreError> {
        let models = client_scopes::Entity::find()
            .filter(client_scopes::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find client scopes by realm id: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(ClientScope::from).collect())
    }

    async fn find_by_name(
        &self,
        name: String,
        realm_id: RealmId,
    ) -> Result<Option<ClientScope>, CoreError> {
        let model = client_scopes::Entity::find()
            .filter(client_scopes::Column::Name.eq(name))
            .filter(client_scopes::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find client scope by name: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(ClientScope::from))
    }

    async fn update_by_id(
        &self,
        id: Uuid,
        payload: UpdateClientScopeRequest,
    ) -> Result<ClientScope, CoreError> {
        let model = client_scopes::Entity::find()
            .filter(client_scopes::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find client scope for update: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active: client_scopes::ActiveModel = model.into();

        active.name = match payload.name {
            Some(v) => Set(v),
            None => active.name,
        };
        active.description = match payload.description {
            Some(v) => Set(Some(v)),
            None => active.description,
        };
        active.protocol = match payload.protocol {
            Some(v) => Set(v),
            None => active.protocol,
        };
        active.is_default = match payload.is_default {
            Some(v) => Set(v),
            None => active.is_default,
        };
        active.updated_at = Set(Utc::now().naive_utc());

        let model = active.update(&self.db).await.map_err(|e| {
            tracing::error!("Failed to update client scope: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model.into())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), CoreError> {
        let result = client_scopes::Entity::delete_many()
            .filter(client_scopes::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete client scope: {}", e);
                CoreError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}
